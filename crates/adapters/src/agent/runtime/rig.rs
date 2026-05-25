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

use crate::agent::config::AgentConfig;
use crate::agent::memory::long_term::extractor::LongTermMemoryExtractor;
use crate::agent::memory::long_term::facts::LongTermFactsMemory as FactsMemory;
use crate::agent::memory::long_term::rag::build_rag_index;
use crate::agent::memory::LongTermFactsMemory;
use crate::agent::memory::SessionConversationMemory;
use crate::agent::runtime::hook::{GuardrailHook, WarmmyPromptHook};
use crate::agent::tool::MealLogTool;
use domain::UserId;

const DEFAULT_MAX_TURNS: usize = 4;
const DEFAULT_HISTORY_WINDOW_MESSAGES: usize = 24;
const WARMMY_PREAMBLE: &str = r#"你是 warmmy，一个温暖、专业的对话型饮食助理。

## 核心行为
- 你可以自然聊天，回答营养健康相关问题
- 你会优先使用 Current Facts 中的权威用户画像理解用户偏好、忌口、过敏原和健康期望
- 当用户明确表达自己吃了或喝了具体内容时，调用 meal_log 工具记录饮食
- 不要为了问候、普通聊天、泛泛咨询或没有具体食物的内容调用工具
- 当用户的问题同时包含画像询问、饮食记录、分析或建议时，在同一次回答中完整处理，不要只回答其中一部分

## 工具调用规范
- 调用 meal_log 时，从用户描述中提取结构化的食物列表（名称、数量、单位）
- 合理推断餐次（breakfast/lunch/dinner/snack），不确定时使用 snack
- 工具执行后，基于工具结果给出简洁自然的中文回复

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

    fn build_tool(&self, user_id: &UserId) -> MealLogTool {
        MealLogTool::new(
            user_id.clone(),
            self.meal_command.clone(),
            self.config.rag.clone(),
        )
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
        let tool = self.build_tool(user_id);
        let memory = self.build_memory(user_id);
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
                    .memory(memory)
                    .dynamic_context(rag_top_k, rag_index);
                agent
                    .hook(hook)
                    .tool(tool)
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
                    .memory(memory)
                    .dynamic_context(rag_top_k, rag_index);
                agent
                    .hook(hook)
                    .tool(tool)
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
        let tool = self.build_tool(user_id);
        let memory = self.build_memory(user_id);
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
                    .memory(memory)
                    .dynamic_context(rag_top_k, rag_index);
                let user_input = prompt.clone();
                let mut raw = agent
                    .hook(hook)
                    .tool(tool)
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
                                    yield Ok(text.text);
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    output_len += r.response().len();
                                    yield Ok(r.response().to_string());
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
                let memory_extractor = self.memory_extractor.clone();
                let user_id_for_memory = user_id.clone();
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
                    .memory(memory)
                    .dynamic_context(rag_top_k, rag_index);
                let user_input = prompt.clone();
                let mut raw = agent
                    .hook(hook)
                    .tool(tool)
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
                                    yield Ok(text.text);
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    output_len += r.response().len();
                                    yield Ok(r.response().to_string());
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
