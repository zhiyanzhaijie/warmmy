use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::conversation::{
    ChatMessageRepositoryPort, ConversationReplyStream, SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::UserDietaryContextQueryHandler;
use async_stream::stream;
use futures_util::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::message::ToolChoice;
use rig::providers::{deepseek, openai};
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use serde_json::json;

use crate::agent::config::AgentConfig;
use crate::agent::interaction::{AgentInteractionRequest, AgentInteractionSink};
use crate::agent::memory::long_term::extractor::LongTermMemoryExtractor;
use crate::agent::memory::long_term::facts::LongTermFactsMemory as FactsMemory;
use crate::agent::memory::long_term::rag::build_rag_index;
use crate::agent::memory::LongTermFactsMemory;
use crate::agent::memory::SessionConversationMemory;
use crate::agent::runtime::hook::{GuardrailHook, WarmmyPromptHook};
use crate::agent::tool;
use domain::UserId;

const DEFAULT_MAX_TURNS: usize = 4;
const DEFAULT_HISTORY_WINDOW_MESSAGES: usize = 24;
const WARMMY_PREAMBLE: &str = r#"你是 warmmy，一个温暖、专业的对话型饮食助理。

## 核心行为
- 你可以自然聊天，回答营养健康相关问题
- 你会优先使用 Current Facts 中的权威用户画像理解用户偏好、忌口、过敏原和健康期望
- 当用户明确表达自己吃了或喝了具体内容，或要求为某个具体餐食创建“确认/待确认记录/草稿/确认卡”时，调用 propose_meal_log 工具创建待确认用餐草稿
- 待确认草稿不是正式 meal log；只有用户确认后，才能调用 confirm_meal_log 正式保存
- 用户取消待确认草稿时，调用 reject_meal_log
- 如果某天已经被用户敲定收尾，你只能围绕该日记录做解释、回顾或建议，不要继续为该日创建或确认新的 meal log
- 不要为了问候、普通聊天、泛泛咨询或没有具体食物的内容调用工具
- 当用户的问题同时包含画像询问、饮食记录、分析或建议时，在同一次回答中完整处理，不要只回答其中一部分

## 工具调用规范
- 调用 propose_meal_log 时，从用户描述中提取结构化的食物列表（名称、数量、单位）；数量或单位缺失时，按常识推断为 1 份
- 如果用户要求“创建确认/晚饭确认/待确认记录/确认卡”，这已经是创建待确认草稿的明确指令，必须调用 propose_meal_log；不要在工具调用前再问“是否确认/是否记录”
- 如果你判断用户在记录一次真实用餐，必须先调用 propose_meal_log；不能只用自然语言让用户确认，也不能自己伪造待确认卡片或 pending_id
- 待确认卡片只能由 propose_meal_log 的工具结果创建；如果没有调用工具，就不要说“请确认这条记录”
- 如果工具返回当天已敲定不能记录的错误，直接向用户解释该日已收尾，建议用户切到正确日期或只继续追问细节
- 合理推断餐次（breakfast/lunch/dinner/snack），不确定时使用 snack
- propose_meal_log 执行后，只能说明“识别到待确认记录”，必须等待用户确认，不要声称已经正式记录
- propose_meal_log 的工具结果如果包含 recorded=false 或 NOT_SAVED_REQUIRES_USER_CONFIRMATION，代表尚未保存；此时绝不能使用“已记录”“已保存”“记下了”等表达，只能请用户确认
- 只有 confirm_meal_log 工具成功返回 status=saved 后，才可以说正式记录成功
- 当收到用户确认待确认记录的 continuation 时，必须调用 confirm_meal_log，再基于工具结果给出最终回复
- 当收到用户取消待确认记录的 continuation 时，必须调用 reject_meal_log，再基于工具结果给出最终回复

## 语言
- 始终使用中文回复"#;

pub struct RigConversationRuntime {
    config: AgentConfig,
    repo: Arc<dyn ChatMessageRepositoryPort>,
    meal_command: Arc<MealCommandHandler>,
    long_term_facts: LongTermFactsMemory,
    memory_extractor: LongTermMemoryExtractor,
    guardrail: Arc<GuardrailHook>,
}

impl RigConversationRuntime {
    pub fn new(
        config: AgentConfig,
        meal_command: Arc<MealCommandHandler>,
        repo: Arc<dyn ChatMessageRepositoryPort>,
        user_contexts: UserDietaryContextQueryHandler,
    ) -> Self {
        let memory_extractor = LongTermMemoryExtractor::new(config.clone());
        Self {
            config,
            repo,
            meal_command,
            long_term_facts: LongTermFactsMemory::new(user_contexts),
            memory_extractor,
            guardrail: Arc::new(GuardrailHook),
        }
    }

    fn build_memory(&self, user_id: &UserId) -> SessionConversationMemory {
        SessionConversationMemory::new(
            user_id.clone(),
            self.repo.clone(),
            DEFAULT_HISTORY_WINDOW_MESSAGES,
        )
    }

    pub async fn complete(
        &self,
        user_id: &UserId,
        session_id: &str,
        prompt: &str,
    ) -> AppResult<SendUserMessageResult> {
        let facts = self.profile_facts(user_id).await;
        let preamble = self.runtime_preamble(&facts);
        let model = self.config.model.as_str();
        let interaction_sink = AgentInteractionSink::default();
        let rag_top_k = self.config.rag.top_k;

        let reply = match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                let rag_index = build_rag_index(&self.config.rag).await?;
                let agent = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(self.build_memory(user_id))
                    .dynamic_context(rag_top_k, rag_index);
                agent
                    .hook(hook)
                    .tools(tool::tools(
                        user_id,
                        session_id,
                        self.meal_command.clone(),
                        interaction_sink.clone(),
                    ))
                    .build()
                    .prompt(prompt)
                    .conversation(session_id)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))?
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                let rag_index = build_rag_index(&self.config.rag).await?;
                let agent = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(self.build_memory(user_id))
                    .dynamic_context(rag_top_k, rag_index);
                agent
                    .hook(hook)
                    .tools(tool::tools(
                        user_id,
                        session_id,
                        self.meal_command.clone(),
                        interaction_sink.clone(),
                    ))
                    .build()
                    .prompt(prompt)
                    .conversation(session_id)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))?
            }
            p => return Err(AppError::internal(format!("unsupported provider: {p}"))),
        };

        if let Err(err) = self
            .memory_extractor
            .extract_and_embed(user_id, prompt)
            .await
        {
            tracing::warn!(error = %err, "failed to extract long-term semantic memory");
        }

        Ok(SendUserMessageResult {
            reply,
            session_id: session_id.to_string(),
        })
    }

    pub async fn stream(
        &self,
        user_id: &UserId,
        session_id: &str,
        prompt: String,
    ) -> AppResult<ConversationReplyStream> {
        let facts = self.profile_facts(user_id).await;
        let preamble = self.runtime_preamble(&facts);
        let model = self.config.model.as_str();
        let interaction_sink = AgentInteractionSink::default();
        let rag_top_k = self.config.rag.top_k;
        let memory_extractor = self.memory_extractor.clone();
        let user_id_for_memory = user_id.clone();

        match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                let rag_index = build_rag_index(&self.config.rag).await?;
                let agent = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(self.build_memory(user_id))
                    .dynamic_context(rag_top_k, rag_index);
                let user_input = prompt.clone();
                let mut raw = agent
                    .hook(hook)
                    .tools(tool::tools(
                        user_id,
                        session_id,
                        self.meal_command.clone(),
                        interaction_sink.clone(),
                    ))
                    .build()
                    .stream_prompt(prompt)
                    .conversation(session_id.to_string())
                    .await;
                let s = stream! {
                    let mut has_text_delta = false;
                    let mut output_len = 0usize;
                    while let Some(item) = raw.next().await {
                        match item {
                            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                                if !text.text.is_empty() {
                                    has_text_delta = true;
                                    output_len += text.text.len();
                                    yield Ok(chat_text_event(&text.text));
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    output_len += r.response().len();
                                    yield Ok(chat_text_event(r.response()));
                                }
                                for interaction in interaction_sink.drain() {
                                    yield Ok(interaction_event(interaction));
                                }
                                if let Err(err) = memory_extractor
                                    .extract_and_embed(&user_id_for_memory, &user_input)
                                    .await
                                {
                                    tracing::warn!(error = %err, "failed to extract long-term semantic memory");
                                }
                                tracing::info!(output.len = output_len, "agent stream finished");
                            }
                            Ok(_) => {}
                            Err(e) => { yield Err(AppError::upstream(e.to_string())); break; }
                        }
                    }
                };
                Ok(Box::pin(s))
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                let rag_index = build_rag_index(&self.config.rag).await?;
                let agent = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(self.build_memory(user_id))
                    .dynamic_context(rag_top_k, rag_index);
                let user_input = prompt.clone();
                let mut raw = agent
                    .hook(hook)
                    .tools(tool::tools(
                        user_id,
                        session_id,
                        self.meal_command.clone(),
                        interaction_sink.clone(),
                    ))
                    .build()
                    .stream_prompt(prompt)
                    .conversation(session_id.to_string())
                    .await;
                let s = stream! {
                    let mut has_text_delta = false;
                    let mut output_len = 0usize;
                    while let Some(item) = raw.next().await {
                        match item {
                            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                                if !text.text.is_empty() {
                                    has_text_delta = true;
                                    output_len += text.text.len();
                                    yield Ok(chat_text_event(&text.text));
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    output_len += r.response().len();
                                    yield Ok(chat_text_event(r.response()));
                                }
                                for interaction in interaction_sink.drain() {
                                    yield Ok(interaction_event(interaction));
                                }
                                if let Err(err) = memory_extractor
                                    .extract_and_embed(&user_id_for_memory, &user_input)
                                    .await
                                {
                                    tracing::warn!(error = %err, "failed to extract long-term semantic memory");
                                }
                                tracing::info!(output.len = output_len, "agent stream finished");
                            }
                            Ok(_) => {}
                            Err(e) => { yield Err(AppError::upstream(e.to_string())); break; }
                        }
                    }
                };
                Ok(Box::pin(s))
            }
            p => Err(AppError::internal(format!("unsupported provider: {p}"))),
        }
    }

    async fn profile_facts(&self, user_id: &UserId) -> String {
        let snapshot = self.long_term_facts.load_profile_snapshot(user_id).await;
        FactsMemory::build_profile_context(snapshot.as_ref())
    }

    fn runtime_preamble(&self, facts: &str) -> String {
        format!("{WARMMY_PREAMBLE}\n\n## Current Facts\n{facts}")
    }
}

fn chat_text_event(text: &str) -> String {
    json!({
        "type": "text_delta",
        "text": text,
    })
    .to_string()
        + "\n"
}

fn interaction_event(interaction: AgentInteractionRequest) -> String {
    json!({
        "type": "interaction_requested",
        "interaction": interaction,
    })
    .to_string()
        + "\n"
}
