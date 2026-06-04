use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::conversation::{
    AgentStatusKind, ChatMessageRepositoryPort, ConversationReplyStream, ConversationStreamEvent,
    ConversationUserInput, EphemeralImageData, EphemeralImageStorePort, SaveMessageImageAttachment,
    SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::{ResolvedAIModelConfig, UserAIConfigQueryHandler, UserDietaryContextQueryHandler};
use async_stream::stream;
use base64::Engine;
use futures_util::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::agent::StreamingError;
use rig::client::CompletionClient;
use rig::completion::Message;
use rig::completion::Prompt;
use rig::message::ToolChoice;
use rig::message::{ImageDetail, ImageMediaType, MimeType, UserContent};
use rig::providers::{deepseek, openai};
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use rig::OneOrMany;
use serde_json::{to_value, Value};
use tokio::sync::mpsc;

use crate::agent::interaction::{AgentInteractionRequest, AgentInteractionSink};
use crate::agent::memory::long_term::extractor::{MemoryObservation, ModelMemoryExtractor};
use crate::agent::memory::long_term::index::NoopMemoryIndex;
use crate::agent::memory::long_term::pipeline::MemoryPipeline;
use crate::agent::memory::long_term::policy::MemoryPolicy;
use crate::agent::memory::long_term::rag::{build_rag_index, LanceDbMemoryIndex, RagConfig};
use crate::agent::memory::long_term::retriever::MemoryRetriever;
use crate::agent::memory::long_term::service::MemoryService;
use crate::agent::memory::long_term::store::MemoryStore;
use crate::agent::memory::MemoryContextProvider;
use crate::agent::memory::SessionConversationMemory;
use crate::agent::runtime::hook::{AgentStatusSink, GuardrailHook, WarmmyPromptHook};
use crate::agent::tool;
use domain::{AICapability, UserId};

const DEFAULT_MAX_TURNS: usize = 4;
const DEFAULT_HISTORY_WINDOW_MESSAGES: usize = 24;
const DEFAULT_EMBEDDING_NDIMS: usize = 1024;
const INTERNAL_CONVERSATION_MARKER: &str = "[warmmy:internal-continuation]";
const WARMMY_PREAMBLE: &str = r#"你是 warmmy，一个温暖、专业的对话型饮食助理。

## 核心行为
- 你可以自然聊天，回答营养健康相关问题
- 你会优先使用 Current Context 理解用户偏好、忌口、过敏原和健康期望
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

enum AgentStreamStep<T> {
    Raw(Option<T>),
    Status(Option<ConversationStreamEvent>),
}

#[derive(Clone)]
pub struct RigConversationRuntime {
    repo: Arc<dyn ChatMessageRepositoryPort>,
    image_store: Arc<dyn EphemeralImageStorePort>,
    memory_store: Arc<dyn MemoryStore>,
    meal_command: Arc<MealCommandHandler>,
    context_provider: MemoryContextProvider,
    ai_configs: UserAIConfigQueryHandler,
    lancedb_path: String,
    rag_top_k: usize,
    guardrail: Arc<GuardrailHook>,
}

impl RigConversationRuntime {
    pub fn new(
        meal_command: Arc<MealCommandHandler>,
        repo: Arc<dyn ChatMessageRepositoryPort>,
        image_store: Arc<dyn EphemeralImageStorePort>,
        memory_store: Arc<dyn MemoryStore>,
        user_contexts: UserDietaryContextQueryHandler,
        ai_configs: UserAIConfigQueryHandler,
        lancedb_path: String,
        rag_top_k: usize,
    ) -> Self {
        Self {
            repo,
            image_store,
            memory_store,
            meal_command,
            context_provider: MemoryContextProvider::new(user_contexts),
            ai_configs,
            lancedb_path,
            rag_top_k,
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

    async fn resolve_conversation_model(
        &self,
        user_id: &UserId,
        input: &ConversationUserInput,
    ) -> AppResult<ResolvedAIModelConfig> {
        let capability = if input.has_images() {
            AICapability::Vision
        } else {
            AICapability::Chat
        };
        self.ai_configs.resolve(user_id, capability).await
    }

    pub async fn complete(
        &self,
        user_id: &UserId,
        session_id: &str,
        input: ConversationUserInput,
    ) -> AppResult<SendUserMessageResult> {
        let chat = self.resolve_conversation_model(user_id, &input).await?;
        let rag = self.resolve_rag(user_id).await?;
        let context = self.context_provider.load(user_id).await;
        let preamble = self.runtime_preamble(&context, rag.is_some());
        let model = chat.model.as_str();
        let interaction_sink = AgentInteractionSink::default();
        let memory_input = input.visible_text();
        let prompt = self.build_prompt_message(&input).await?;
        self.persist_user_image_message(user_id, session_id, &input, &memory_input)
            .await;
        self.persist_user_visible_message(user_id, session_id, &input, &memory_input)
            .await?;

        let reply = match chat.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&chat.api_key)
                    .base_url(&chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                if let Some(rag) = rag.clone() {
                    let rag_index = self.build_memory_retriever(user_id, &rag).await?;
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .dynamic_context(rag.top_k, rag_index)
                        .hook(hook)
                        .tools(tool::tools(
                            user_id,
                            session_id,
                            self.meal_command.clone(),
                            interaction_sink.clone(),
                        ))
                        .build()
                        .prompt(prompt.clone())
                        .conversation(session_id)
                        .await
                        .map_err(|e| AppError::upstream(e.to_string()))?
                } else {
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
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
            }
            "openai_compatible" | "siliconflow" => {
                let client = openai::Client::builder()
                    .api_key(&chat.api_key)
                    .base_url(&chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?
                    .completions_api();
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                if let Some(rag) = rag.clone() {
                    let rag_index = self.build_memory_retriever(user_id, &rag).await?;
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .dynamic_context(rag.top_k, rag_index)
                        .hook(hook)
                        .tools(tool::tools(
                            user_id,
                            session_id,
                            self.meal_command.clone(),
                            interaction_sink.clone(),
                        ))
                        .build()
                        .prompt(prompt.clone())
                        .conversation(session_id)
                        .await
                        .map_err(|e| AppError::upstream(e.to_string()))?
                } else {
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
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
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&chat.api_key)
                    .base_url(&chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                if let Some(rag) = rag.clone() {
                    let rag_index = self.build_memory_retriever(user_id, &rag).await?;
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .dynamic_context(rag.top_k, rag_index)
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
                } else {
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
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
            }
            p => return Err(AppError::internal(format!("unsupported provider: {p}"))),
        };

        self.persist_assistant_visible_message(user_id, session_id, &reply)
            .await?;
        self.observe_memory(user_id, session_id, &memory_input, &reply)
            .await;

        Ok(SendUserMessageResult {
            reply,
            session_id: session_id.to_string(),
        })
    }

    pub async fn stream(
        &self,
        user_id: &UserId,
        session_id: &str,
        input: ConversationUserInput,
    ) -> AppResult<ConversationReplyStream> {
        let chat = self.resolve_conversation_model(user_id, &input).await?;
        let runtime = self.clone();
        let user_id = user_id.clone();
        let session_id = session_id.to_string();
        let has_images = input.has_images();

        Ok(Box::pin(stream! {
            yield Ok(stream_event(ConversationStreamEvent::RunStarted {
                run_id: format!("{user_id}:{session_id}"),
            }));
            if has_images {
                yield Ok(status_event(
                    AgentStatusKind::ReadingInput,
                    "我在认真看看这张图片。",
                ));
            } else {
                yield Ok(status_event(
                    AgentStatusKind::Thinking,
                    "我在准备这次对话的上下文。",
                ));
            }

            match runtime
                .open_stream_with_model(&user_id, &session_id, input, chat)
                .await
            {
                Ok(mut reply_stream) => {
                    while let Some(item) = reply_stream.next().await {
                        yield item;
                    }
                }
                Err(err) => yield Err(err),
            }
        }))
    }

    async fn open_stream_with_model(
        &self,
        user_id: &UserId,
        session_id: &str,
        input: ConversationUserInput,
        chat: ResolvedAIModelConfig,
    ) -> AppResult<ConversationReplyStream> {
        let rag = self.resolve_rag(user_id).await?;
        let context = self.context_provider.load(user_id).await;
        let preamble = self.runtime_preamble(&context, rag.is_some());
        let model = chat.model.as_str();
        let interaction_sink = AgentInteractionSink::default();
        let user_input = input.visible_text();
        let has_images = input.has_images();
        let prompt = self.build_prompt_message(&input).await?;
        self.persist_user_image_message(user_id, session_id, &input, &user_input)
            .await;
        self.persist_user_visible_message(user_id, session_id, &input, &user_input)
            .await?;
        let user_id_for_history = user_id.clone();
        let user_id_for_memory = user_id.clone();
        let session_id_for_history = session_id.to_string();
        let session_id_for_memory = session_id.to_string();
        let repo_for_history = self.repo.clone();
        let runtime_for_memory = self.clone();
        let user_input_for_memory = user_input.clone();
        let (status_tx, status_rx) = mpsc::unbounded_channel();
        let status_sink = AgentStatusSink::new(status_tx);

        match chat.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&chat.api_key)
                    .base_url(&chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook =
                    WarmmyPromptHook::with_status_sink(self.guardrail.clone(), status_sink.clone());
                let raw = if let Some(rag) = rag.clone() {
                    let rag_index = self.build_memory_retriever(user_id, &rag).await?;
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .dynamic_context(rag.top_k, rag_index)
                        .hook(hook)
                        .tools(tool::tools(
                            user_id,
                            session_id,
                            self.meal_command.clone(),
                            interaction_sink.clone(),
                        ))
                        .build()
                        .stream_prompt(prompt.clone())
                        .conversation(session_id.to_string())
                        .await
                } else {
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .hook(hook)
                        .tools(tool::tools(
                            user_id,
                            session_id,
                            self.meal_command.clone(),
                            interaction_sink.clone(),
                        ))
                        .build()
                        .stream_prompt(prompt.clone())
                        .conversation(session_id.to_string())
                        .await
                };
                Ok(Self::wrap_stream(
                    raw,
                    interaction_sink,
                    has_images,
                    repo_for_history.clone(),
                    user_id_for_history.clone(),
                    session_id_for_history.clone(),
                    runtime_for_memory.clone(),
                    user_id_for_memory.clone(),
                    session_id_for_memory.clone(),
                    user_input_for_memory.clone(),
                    status_rx,
                ))
            }
            "openai_compatible" | "siliconflow" => {
                let client = openai::Client::builder()
                    .api_key(&chat.api_key)
                    .base_url(&chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?
                    .completions_api();
                let hook =
                    WarmmyPromptHook::with_status_sink(self.guardrail.clone(), status_sink.clone());
                let raw = if let Some(rag) = rag.clone() {
                    let rag_index = self.build_memory_retriever(user_id, &rag).await?;
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .dynamic_context(rag.top_k, rag_index)
                        .hook(hook)
                        .tools(tool::tools(
                            user_id,
                            session_id,
                            self.meal_command.clone(),
                            interaction_sink.clone(),
                        ))
                        .build()
                        .stream_prompt(prompt.clone())
                        .conversation(session_id.to_string())
                        .await
                } else {
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .hook(hook)
                        .tools(tool::tools(
                            user_id,
                            session_id,
                            self.meal_command.clone(),
                            interaction_sink.clone(),
                        ))
                        .build()
                        .stream_prompt(prompt.clone())
                        .conversation(session_id.to_string())
                        .await
                };
                Ok(Self::wrap_stream(
                    raw,
                    interaction_sink,
                    has_images,
                    repo_for_history.clone(),
                    user_id_for_history.clone(),
                    session_id_for_history.clone(),
                    runtime_for_memory.clone(),
                    user_id_for_memory.clone(),
                    session_id_for_memory.clone(),
                    user_input_for_memory.clone(),
                    status_rx,
                ))
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&chat.api_key)
                    .base_url(&chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook =
                    WarmmyPromptHook::with_status_sink(self.guardrail.clone(), status_sink.clone());
                let raw = if let Some(rag) = rag.clone() {
                    let rag_index = self.build_memory_retriever(user_id, &rag).await?;
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
                        .dynamic_context(rag.top_k, rag_index)
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
                        .await
                } else {
                    client
                        .agent(model)
                        .preamble(&preamble)
                        .tool_choice(ToolChoice::Auto)
                        .default_max_turns(DEFAULT_MAX_TURNS)
                        .memory(self.build_memory(user_id))
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
                        .await
                };
                Ok(Self::wrap_stream(
                    raw,
                    interaction_sink,
                    has_images,
                    repo_for_history,
                    user_id_for_history,
                    session_id_for_history,
                    runtime_for_memory,
                    user_id_for_memory,
                    session_id_for_memory,
                    user_input_for_memory,
                    status_rx,
                ))
            }
            p => Err(AppError::internal(format!("unsupported provider: {p}"))),
        }
    }

    async fn persist_user_visible_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        input: &ConversationUserInput,
        visible_text: &str,
    ) -> AppResult<()> {
        let visible_text = visible_text.trim();
        if visible_text.is_empty()
            || input.has_images()
            || is_internal_conversation_input(visible_text)
        {
            return Ok(());
        }

        self.repo
            .save_message(user_id, session_id, "user", visible_text)
            .await
            .map_err(AppError::database)?;
        Ok(())
    }

    async fn persist_assistant_visible_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        reply: &str,
    ) -> AppResult<()> {
        let reply = reply.trim();
        if reply.is_empty() {
            return Ok(());
        }

        self.repo
            .save_message(user_id, session_id, "assistant", reply)
            .await
            .map_err(AppError::database)?;
        Ok(())
    }

    async fn build_prompt_message(&self, input: &ConversationUserInput) -> AppResult<Message> {
        let images = self.load_image_data(input).await?;
        let mut content = Vec::new();
        let text = input.text.trim();

        if !text.is_empty() || images.is_empty() {
            content.push(UserContent::text(text.to_string()));
        }

        for image in images {
            let media_type = ImageMediaType::from_mime_type(&image.mime_type).ok_or_else(|| {
                AppError::validation(format!("unsupported image mime type: {}", image.mime_type))
            })?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(image.bytes);
            content.push(UserContent::image_base64(
                encoded,
                Some(media_type),
                Some(ImageDetail::Auto),
            ));
        }

        let content = OneOrMany::many(content)
            .map_err(|_| AppError::validation("empty conversation input"))?;
        Ok(Message::User { content })
    }

    async fn load_image_data(
        &self,
        input: &ConversationUserInput,
    ) -> AppResult<Vec<EphemeralImageData>> {
        let mut images = Vec::new();
        for image in input.image_attachments() {
            images.push(self.image_store.load_image(&image.asset_id).await?);
        }
        Ok(images)
    }

    async fn persist_user_image_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        input: &ConversationUserInput,
        visible_text: &str,
    ) {
        let attachments = input
            .image_attachments()
            .map(|image| SaveMessageImageAttachment {
                mime_type: image.mime_type.clone(),
                size_bytes: image.size_bytes,
                width: image.width,
                height: image.height,
                data_url: image.preview_data_url.clone(),
                status: if image.preview_data_url.is_some() {
                    "available".to_string()
                } else {
                    "missing".to_string()
                },
            })
            .collect::<Vec<_>>();

        if attachments.is_empty() {
            return;
        }

        if let Err(err) = self
            .repo
            .save_message_with_attachments(user_id, session_id, "user", visible_text, attachments)
            .await
        {
            tracing::warn!(error = %err, "failed to persist image message attachments");
        }
    }

    fn wrap_stream<S, R>(
        mut raw: S,
        interaction_sink: AgentInteractionSink,
        has_images: bool,
        repo_for_history: Arc<dyn ChatMessageRepositoryPort>,
        user_id_for_history: UserId,
        session_id_for_history: String,
        runtime_for_memory: RigConversationRuntime,
        user_id_for_memory: UserId,
        session_id_for_memory: String,
        user_input_for_memory: String,
        mut status_rx: mpsc::UnboundedReceiver<ConversationStreamEvent>,
    ) -> ConversationReplyStream
    where
        S: futures_core::Stream<Item = Result<MultiTurnStreamItem<R>, StreamingError>>
            + Send
            + Unpin
            + 'static,
        R: Clone + Send + 'static,
    {
        let s = stream! {
            yield Ok(status_event(
                AgentStatusKind::CallingModel,
                if has_images {
                    "我看完图片了，正在组织回复。"
                } else {
                    "我在组织回复。"
                },
            ));
            let mut has_text_delta = false;
            let mut assistant_output = String::new();
            let mut output_len = 0usize;
            let mut status_open = true;
            loop {
                let step = if status_open {
                    tokio::select! {
                        event = status_rx.recv() => AgentStreamStep::Status(event),
                        item = raw.next() => AgentStreamStep::Raw(item),
                    }
                } else {
                    AgentStreamStep::Raw(raw.next().await)
                };

                let item = match step {
                    AgentStreamStep::Status(Some(event)) => {
                        yield Ok(stream_event(event));
                        continue;
                    }
                    AgentStreamStep::Status(None) => {
                        status_open = false;
                        continue;
                    }
                    AgentStreamStep::Raw(Some(item)) => item,
                    AgentStreamStep::Raw(None) => break,
                };

                match item {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                        if !text.text.is_empty() {
                            has_text_delta = true;
                            output_len += text.text.len();
                            assistant_output.push_str(&text.text);
                            yield Ok(stream_event(ConversationStreamEvent::TextDelta {
                                text: text.text,
                            }));
                        }
                    }
                    Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                        if !has_text_delta && !r.response().is_empty() {
                            output_len += r.response().len();
                            assistant_output.push_str(r.response());
                            yield Ok(stream_event(ConversationStreamEvent::TextDelta {
                                text: r.response().to_string(),
                            }));
                        }
                        for interaction in interaction_sink.drain() {
                            yield Ok(interaction_event(interaction));
                        }
                        yield Ok(status_event(
                            AgentStatusKind::Persisting,
                            "我在保存这次对话。",
                        ));
                        if let Err(err) = persist_assistant_visible_message(
                            &repo_for_history,
                            &user_id_for_history,
                            &session_id_for_history,
                            &assistant_output,
                        )
                        .await
                        {
                            yield Err(err);
                            break;
                        }
                        while let Ok(event) = status_rx.try_recv() {
                            yield Ok(stream_event(event));
                        }
                        let runtime_for_memory = runtime_for_memory.clone();
                        let user_id_for_memory = user_id_for_memory.clone();
                        let session_id_for_memory = session_id_for_memory.clone();
                        let user_input_for_memory = user_input_for_memory.clone();
                        let assistant_output_for_memory = assistant_output.clone();
                        tokio::spawn(async move {
                            runtime_for_memory
                                .observe_memory(
                                    &user_id_for_memory,
                                    &session_id_for_memory,
                                    &user_input_for_memory,
                                    &assistant_output_for_memory,
                                )
                                .await;
                        });
                        yield Ok(stream_event(ConversationStreamEvent::Done));
                        tracing::info!(output.len = output_len, "agent stream finished");
                    }
                    Ok(_) => {}
                    Err(e) => {
                        yield Err(AppError::upstream(e.to_string()));
                        break;
                    }
                }
            }
        };
        Box::pin(s)
    }

    async fn resolve_rag(&self, user_id: &UserId) -> AppResult<Option<RagConfig>> {
        let Some(embedding) = self
            .ai_configs
            .resolve_optional(user_id, AICapability::Embedding)
            .await?
        else {
            tracing::info!(
                user.id = user_id.as_str(),
                "semantic memory disabled: embedding config missing"
            );
            return Ok(None);
        };

        Ok(Some(RagConfig {
            lancedb_path: self.lancedb_path.clone(),
            embedding_base_url: embedding.base_url,
            embedding_api_key: embedding.api_key,
            embedding_model: embedding.model,
            embedding_ndims: embedding.embedding_ndims.unwrap_or(DEFAULT_EMBEDDING_NDIMS),
            top_k: self.rag_top_k,
        }))
    }

    async fn build_memory_retriever(
        &self,
        user_id: &UserId,
        rag: &RagConfig,
    ) -> AppResult<MemoryRetriever> {
        let index = build_rag_index(rag).await?;
        Ok(MemoryRetriever::new(
            user_id.clone(),
            self.memory_store.clone(),
            index,
        ))
    }

    async fn observe_memory(
        &self,
        user_id: &UserId,
        session_id: &str,
        user_input: &str,
        assistant_output: &str,
    ) {
        if user_input.trim().is_empty() || is_internal_conversation_input(user_input) {
            return;
        }

        let extractor_model = match self
            .ai_configs
            .resolve_optional(user_id, AICapability::Chat)
            .await
        {
            Ok(Some(model)) => model,
            Ok(None) => {
                tracing::info!(
                    user.id = user_id.as_str(),
                    "memory observation skipped: chat model config missing"
                );
                return;
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to resolve chat model for memory observation");
                return;
            }
        };

        let memory_index: Arc<dyn crate::agent::memory::long_term::index::MemoryIndex> =
            match self.resolve_rag(user_id).await {
                Ok(Some(rag)) => Arc::new(LanceDbMemoryIndex::new(rag)),
                Ok(None) => Arc::new(NoopMemoryIndex),
                Err(err) => {
                    tracing::warn!(error = %err, "failed to resolve memory index");
                    Arc::new(NoopMemoryIndex)
                }
            };

        let pipeline = MemoryPipeline::new(
            Arc::new(ModelMemoryExtractor::new(extractor_model)),
            MemoryService::new(
                MemoryPolicy::new(),
                self.memory_store.clone(),
                memory_index,
            ),
        );

        if let Err(err) = pipeline
            .observe(MemoryObservation {
                user_id: user_id.clone(),
                session_id: session_id.to_string(),
                reference_date: session_id.to_string(),
                user_input: user_input.to_string(),
                assistant_output: assistant_output.to_string(),
            })
            .await
        {
            tracing::warn!(error = %err, "failed to observe memory");
        }
    }

    fn runtime_preamble(&self, context: &str, semantic_memory_enabled: bool) -> String {
        let semantic_status = if semantic_memory_enabled {
            "长期语义记忆/RAG 已启用。"
        } else {
            "长期语义记忆/RAG 未启用：用户尚未配置 embedding 模型或 API key。只能使用当前会话记忆和 Current Context。"
        };
        format!("{WARMMY_PREAMBLE}\n\n## Capability Status\n{semantic_status}\n\n## Current Context\n{context}")
    }
}

fn status_event(kind: AgentStatusKind, label: &str) -> String {
    stream_event(ConversationStreamEvent::Status {
        kind,
        label: label.to_string(),
    })
}

fn interaction_event(interaction: AgentInteractionRequest) -> String {
    let interaction = to_value(interaction).unwrap_or(Value::Null);
    stream_event(ConversationStreamEvent::InteractionRequested { interaction })
}

fn stream_event(event: ConversationStreamEvent) -> String {
    match serde_json::to_string(&event) {
        Ok(line) => line + "\n",
        Err(err) => {
            tracing::warn!(error = %err, "failed to serialize conversation stream event");
            String::new()
        }
    }
}

async fn persist_assistant_visible_message(
    repo: &Arc<dyn ChatMessageRepositoryPort>,
    user_id: &UserId,
    session_id: &str,
    reply: &str,
) -> AppResult<()> {
    let reply = reply.trim();
    if reply.is_empty() {
        return Ok(());
    }

    repo.save_message(user_id, session_id, "assistant", reply)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

fn is_internal_conversation_input(text: &str) -> bool {
    text.trim_start().starts_with(INTERNAL_CONVERSATION_MARKER)
        || text.starts_with("用户已在界面确认一条待确认用餐记录。")
        || text.starts_with("用户已在界面取消一条待确认用餐记录。")
}
