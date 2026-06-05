use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::conversation::{
    AgentStatusKind, ChatMessage, ChatMessageRepositoryPort, ConversationReplyStream,
    ConversationStreamEvent, ConversationSummary, ConversationUserInput, EphemeralImageData,
    EphemeralImageStorePort, SaveMessageImageAttachment, SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::{ResolvedAIModelConfig, UserAIConfigQueryHandler, UserDietaryContextQueryHandler};
use async_stream::stream;
use base64::Engine;
use futures_util::StreamExt;
use rig::agent::{Agent, AgentBuilder, MultiTurnStreamItem, StreamingError};
use rig::client::CompletionClient;
use rig::completion::{CompletionModel, Message, Prompt};
use rig::message::ToolChoice;
use rig::message::{ImageDetail, ImageMediaType, MimeType, UserContent};
use rig::providers::{deepseek, openai};
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use rig::tool::ToolDyn;
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
use crate::agent::prompts::conversation_summary::CONVERSATION_SUMMARY_PREAMBLE;
use crate::agent::routing::classifier::{classify_route, AgentRoute};
use crate::agent::runtime::hook::{AgentStatusSink, GuardrailHook, WarmmyPromptHook};
use crate::agent::services::nutrition::curator::{ModelNutritionCurator, NutritionCurator};
use crate::agent::services::nutrition::retriever::{
    LanceDbNutritionReferenceRetriever, NutritionReferenceRetriever,
};
use crate::agent::services::AgentServiceProgress;
use crate::agent::tool;
use domain::{AICapability, UserId};

const DEFAULT_MAX_TURNS: usize = 4;
const DEFAULT_HISTORY_WINDOW_MESSAGES: usize = 16;
const DEFAULT_EMBEDDING_NDIMS: usize = 1024;
const INTERNAL_CONVERSATION_MARKER: &str = "[warmmy:internal-continuation]";
const WARMMY_PREAMBLE: &str = r#"你是 warmmy，一个温暖、专业的对话型饮食助理。

## 自我认知
- 你可以自然聊天，回答营养健康相关问题
- 你会优先使用 Current Context 理解用户偏好、忌口、过敏原和健康期望
- 当用户的问题同时包含画像询问、饮食记录、分析或建议时，在同一次回答中完整处理，不要只回答其中一部分

## 语言
- 始终使用中文回复"#;

enum AgentStreamStep<T> {
    Raw(Option<T>),
    Status(Option<ConversationStreamEvent>),
}

struct StreamAgentServiceProgress {
    sink: AgentStatusSink,
}

impl AgentServiceProgress for StreamAgentServiceProgress {
    fn status(&self, label: &'static str) {
        self.sink.emit(ConversationStreamEvent::Status {
            kind: AgentStatusKind::UsingTool,
            label: label.to_string(),
        });
    }
}

struct RagAgentContext {
    top_k: usize,
    index: MemoryRetriever,
}

struct AgentSpec {
    preamble: String,
    tool_choice: ToolChoice,
    memory: SessionConversationMemory,
    tools: Vec<Box<dyn ToolDyn>>,
    rag: Option<RagAgentContext>,
    guardrail: Arc<GuardrailHook>,
    status_sink: Option<AgentStatusSink>,
}

type OpenAiResponsesModel = openai::responses_api::ResponsesCompletionModel;
type OpenAiCompletionsModel = openai::completion::CompletionModel;
type DeepSeekModel = deepseek::CompletionModel;

type WarmmyAgent<M> = Agent<M, WarmmyPromptHook<M>>;

enum ConversationAgent {
    OpenAiResponses(WarmmyAgent<OpenAiResponsesModel>),
    OpenAiCompletions(WarmmyAgent<OpenAiCompletionsModel>),
    DeepSeek(WarmmyAgent<DeepSeekModel>),
}

impl ConversationAgent {
    async fn prompt(self, prompt: Message, conversation_id: &str) -> AppResult<String> {
        match self {
            Self::OpenAiResponses(agent) => agent
                .prompt(prompt)
                .conversation(conversation_id)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
            Self::OpenAiCompletions(agent) => agent
                .prompt(prompt)
                .conversation(conversation_id)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
            Self::DeepSeek(agent) => agent
                .prompt(prompt)
                .conversation(conversation_id)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
        }
    }
}

struct StreamWrapCtx {
    interaction_sink: AgentInteractionSink,
    has_images: bool,
    repo_for_history: Arc<dyn ChatMessageRepositoryPort>,
    user_id_for_history: UserId,
    session_id_for_history: String,
    runtime_for_memory: RigConversationRuntime,
    user_id_for_memory: UserId,
    session_id_for_memory: String,
    user_input_for_memory: String,
    status_rx: mpsc::UnboundedReceiver<ConversationStreamEvent>,
}

fn configure_agent<M>(builder: AgentBuilder<M>, spec: AgentSpec) -> WarmmyAgent<M>
where
    M: CompletionModel + 'static,
{
    let hook = match spec.status_sink {
        Some(status_sink) => WarmmyPromptHook::with_status_sink(spec.guardrail, status_sink),
        None => WarmmyPromptHook::new(spec.guardrail),
    };

    let mut builder = builder
        .preamble(&spec.preamble)
        .tool_choice(spec.tool_choice)
        .default_max_turns(DEFAULT_MAX_TURNS)
        .memory(spec.memory);

    if let Some(rag) = spec.rag {
        builder = builder.dynamic_context(rag.top_k, rag.index);
    }

    builder.hook(hook).tools(spec.tools).build()
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

    fn nutrition_curator(&self, chat: &ResolvedAIModelConfig) -> Option<Arc<dyn NutritionCurator>> {
        Some(Arc::new(ModelNutritionCurator::new(chat.clone())))
    }

    fn nutrition_retriever(
        &self,
        rag: Option<&RagConfig>,
    ) -> Option<Arc<dyn NutritionReferenceRetriever>> {
        let rag = rag?;
        let references = self.meal_command.food_nutrition_references()?;
        Some(Arc::new(LanceDbNutritionReferenceRetriever::new(
            references,
            rag.clone(),
        )))
    }

    fn agent_service_progress(
        &self,
        status_sink: Option<AgentStatusSink>,
    ) -> Option<Arc<dyn AgentServiceProgress>> {
        status_sink.map(|sink| {
            Arc::new(StreamAgentServiceProgress { sink }) as Arc<dyn AgentServiceProgress>
        })
    }

    fn openai_client(config: &ResolvedAIModelConfig) -> AppResult<openai::Client> {
        openai::Client::builder()
            .api_key(&config.api_key)
            .base_url(&config.base_url)
            .build()
            .map_err(|e| AppError::upstream(e.to_string()))
    }

    fn deepseek_client(config: &ResolvedAIModelConfig) -> AppResult<deepseek::Client> {
        deepseek::Client::builder()
            .api_key(&config.api_key)
            .base_url(&config.base_url)
            .build()
            .map_err(|e| AppError::upstream(e.to_string()))
    }

    async fn build_rag_agent_context(
        &self,
        user_id: &UserId,
        rag: Option<&RagConfig>,
    ) -> AppResult<Option<RagAgentContext>> {
        let Some(rag) = rag else {
            return Ok(None);
        };
        let index = self.build_memory_retriever(user_id, rag).await?;
        Ok(Some(RagAgentContext {
            top_k: rag.top_k,
            index,
        }))
    }

    fn build_agent_spec(
        &self,
        user_id: &UserId,
        session_id: &str,
        chat: &ResolvedAIModelConfig,
        preamble: String,
        route: AgentRoute,
        tool_choice: ToolChoice,
        interaction_sink: AgentInteractionSink,
        nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
        rag: Option<RagAgentContext>,
        status_sink: Option<AgentStatusSink>,
    ) -> AgentSpec {
        AgentSpec {
            preamble,
            tool_choice,
            memory: self.build_memory(user_id),
            tools: tool::tools_for_route(
                route,
                user_id,
                session_id,
                self.meal_command.clone(),
                interaction_sink,
                self.nutrition_curator(chat),
                nutrition_retriever,
                self.agent_service_progress(status_sink.clone()),
            ),
            rag,
            guardrail: self.guardrail.clone(),
            status_sink,
        }
    }

    fn build_conversation_agent(
        &self,
        chat: &ResolvedAIModelConfig,
        spec: AgentSpec,
    ) -> AppResult<ConversationAgent> {
        let model = chat.model.as_str();
        match chat.provider.as_str() {
            "openai" => {
                let client = Self::openai_client(chat)?;
                Ok(ConversationAgent::OpenAiResponses(configure_agent(
                    client.agent(model),
                    spec,
                )))
            }
            "openai_compatible" | "siliconflow" => {
                let client = Self::openai_client(chat)?.completions_api();
                Ok(ConversationAgent::OpenAiCompletions(configure_agent(
                    client.agent(model),
                    spec,
                )))
            }
            "deepseek" => {
                let client = Self::deepseek_client(chat)?;
                Ok(ConversationAgent::DeepSeek(configure_agent(
                    client.agent(model),
                    spec,
                )))
            }
            provider => Err(AppError::internal(format!(
                "unsupported provider: {provider}"
            ))),
        }
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
        let nutrition_retriever = self.nutrition_retriever(rag.as_ref());
        let context = self.context_provider.load(user_id).await;
        let preamble = self.runtime_preamble(&context, true, rag.is_some(), input.has_images());
        let interaction_sink = AgentInteractionSink::default();
        let memory_input = input.visible_text();
        let route_decision = classify_route(&memory_input, input.has_images(), &chat).await?;
        let route = route_decision.route;
        let tool_choice = tool_choice_for_route(route, &memory_input);
        tracing::info!(
            agent.route = ?route,
            route.source = ?route_decision.source,
            route.reason = %route_decision.reason,
            session.id = %session_id,
            input.has_images = input.has_images(),
            "agent route selected"
        );
        let prompt = self.build_prompt_message(&input).await?;
        self.persist_user_image_message(user_id, session_id, &input, &memory_input)
            .await;
        self.persist_user_visible_message(user_id, session_id, &input, &memory_input)
            .await?;

        let rag_context = self.build_rag_agent_context(user_id, rag.as_ref()).await?;
        let spec = self.build_agent_spec(
            user_id,
            session_id,
            &chat,
            preamble,
            route,
            tool_choice,
            interaction_sink,
            nutrition_retriever,
            rag_context,
            None,
        );
        let reply = self
            .build_conversation_agent(&chat, spec)?
            .prompt(prompt, session_id)
            .await?;

        self.persist_assistant_visible_message(user_id, session_id, &reply)
            .await?;
        self.observe_memory(user_id, session_id, &memory_input, &reply)
            .await;
        self.spawn_conversation_summary_update(user_id, session_id);

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
        let visible_text = input.visible_text();
        let route_decision = classify_route(&visible_text, has_images, &chat).await?;
        let route = route_decision.route;
        tracing::info!(
            agent.route = ?route,
            route.source = ?route_decision.source,
            route.reason = %route_decision.reason,
            session.id = %session_id,
            input.has_images = has_images,
            "agent route selected"
        );

        Ok(Box::pin(stream! {
            yield Ok(stream_event(ConversationStreamEvent::RunStarted {
                run_id: format!("{user_id}:{session_id}"),
            }));
            yield Ok(initial_route_status(route, has_images));

            match runtime
                .open_stream_with_model(&user_id, &session_id, input, chat, route)
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
        route: AgentRoute,
    ) -> AppResult<ConversationReplyStream> {
        let rag = self.resolve_rag(user_id).await?;
        let nutrition_retriever = self.nutrition_retriever(rag.as_ref());
        let context = self.context_provider.load(user_id).await;
        let preamble = self.runtime_preamble(&context, true, rag.is_some(), input.has_images());
        let interaction_sink = AgentInteractionSink::default();
        let user_input = input.visible_text();
        let has_images = input.has_images();
        let tool_choice = tool_choice_for_route(route, &user_input);
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

        let rag_context = self.build_rag_agent_context(user_id, rag.as_ref()).await?;
        let spec = self.build_agent_spec(
            user_id,
            session_id,
            &chat,
            preamble,
            route,
            tool_choice,
            interaction_sink.clone(),
            nutrition_retriever,
            rag_context,
            Some(status_sink.clone()),
        );
        let agent = self.build_conversation_agent(&chat, spec)?;
        let wrap_ctx = StreamWrapCtx {
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
        };

        match agent {
            ConversationAgent::OpenAiResponses(agent) => {
                let raw = agent
                    .stream_prompt(prompt)
                    .conversation(session_id.to_string())
                    .await;
                Ok(Self::wrap_stream(raw, wrap_ctx))
            }
            ConversationAgent::OpenAiCompletions(agent) => {
                let raw = agent
                    .stream_prompt(prompt)
                    .conversation(session_id.to_string())
                    .await;
                Ok(Self::wrap_stream(raw, wrap_ctx))
            }
            ConversationAgent::DeepSeek(agent) => {
                let raw = agent
                    .stream_prompt(prompt)
                    .conversation(session_id.to_string())
                    .await;
                Ok(Self::wrap_stream(raw, wrap_ctx))
            }
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

    fn wrap_stream<S, R>(mut raw: S, ctx: StreamWrapCtx) -> ConversationReplyStream
    where
        S: futures_core::Stream<Item = Result<MultiTurnStreamItem<R>, StreamingError>>
            + Send
            + Unpin
            + 'static,
        R: Clone + Send + 'static,
    {
        let StreamWrapCtx {
            interaction_sink,
            has_images,
            repo_for_history,
            user_id_for_history,
            session_id_for_history,
            runtime_for_memory,
            user_id_for_memory,
            session_id_for_memory,
            user_input_for_memory,
            mut status_rx,
        } = ctx;
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
                            tracing::info!(
                                session.id = %session_id_for_history,
                                interaction.id = %interaction.id,
                                interaction.kind = %interaction.kind,
                                "stream interaction emitted"
                            );
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
                            runtime_for_memory.spawn_conversation_summary_update(
                                &user_id_for_memory,
                                &session_id_for_memory,
                            );
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
            MemoryService::new(MemoryPolicy::new(), self.memory_store.clone(), memory_index),
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

    fn spawn_conversation_summary_update(&self, user_id: &UserId, session_id: &str) {
        let runtime = self.clone();
        let user_id = user_id.clone();
        let session_id = session_id.to_string();
        tokio::spawn(async move {
            if let Err(err) = runtime
                .update_conversation_summary(&user_id, &session_id)
                .await
            {
                tracing::warn!(error = %err, "failed to update conversation summary");
            }
        });
    }

    async fn update_conversation_summary(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> AppResult<()> {
        let summary = self
            .repo
            .find_conversation_summary(user_id, session_id)
            .await
            .map_err(AppError::database)?;
        let messages = self
            .repo
            .find_by_session(user_id, session_id)
            .await
            .map_err(AppError::database)?;

        let summarized_until = summary
            .as_ref()
            .map(|summary| summary.summarized_until_index.max(0) as usize)
            .unwrap_or_default();
        let window_end = summarized_until + DEFAULT_HISTORY_WINDOW_MESSAGES;
        if messages.len() <= window_end {
            return Ok(());
        }

        let window = &messages[summarized_until..window_end];
        let Some(last_message) = window.last() else {
            return Ok(());
        };

        let model = match self
            .ai_configs
            .resolve_optional(user_id, AICapability::Chat)
            .await?
        {
            Some(model) => model,
            None => return Ok(()),
        };

        let previous_summary = summary
            .as_ref()
            .map(|summary| summary.summary.as_str())
            .unwrap_or_default();
        let prompt = build_conversation_summary_prompt(previous_summary, window);
        let next_summary = self.prompt_summary_model(&model, prompt).await?;
        if next_summary.trim().is_empty() {
            return Ok(());
        }

        self.repo
            .save_conversation_summary(
                user_id,
                session_id,
                &ConversationSummary {
                    summary: next_summary.trim().to_string(),
                    summarized_until_message_id: Some(last_message.id.clone()),
                    summarized_until_index: window_end as i32,
                },
            )
            .await
            .map_err(AppError::database)?;

        tracing::info!(
            session.id = session_id,
            summarized_until_index = window_end,
            "conversation summary updated"
        );
        Ok(())
    }

    async fn prompt_summary_model(
        &self,
        model: &ResolvedAIModelConfig,
        prompt: String,
    ) -> AppResult<String> {
        match model.provider.as_str() {
            "openai" | "openai_compatible" | "siliconflow" => {
                let client = Self::openai_client(model)?.completions_api();
                client
                    .agent(model.model.as_str())
                    .preamble(CONVERSATION_SUMMARY_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))
            }
            "deepseek" => {
                let client = Self::deepseek_client(model)?;
                client
                    .agent(model.model.as_str())
                    .preamble(CONVERSATION_SUMMARY_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))
            }
            provider => Err(AppError::internal(format!(
                "unsupported summary provider: {provider}"
            ))),
        }
    }

    fn runtime_preamble(
        &self,
        context: &str,
        chat_enabled: bool,
        semantic_memory_enabled: bool,
        vision_enabled: bool,
    ) -> String {
        let chat_status = if chat_enabled {
            "聊天功能已启用。"
        } else {
            "聊天功能未启用。"
        };
        let semantic_status = if semantic_memory_enabled {
            "长期语义记忆/RAG 已启用。"
        } else {
            "长期语义记忆/RAG 未启用：用户尚未配置 embedding 模型或 API key。只能使用当前会话记忆和 Current Context。"
        };
        let vision_status = if vision_enabled {
            "识图功能本轮已启用。"
        } else {
            "识图功能本轮未启用。"
        };
        format!(
            "{WARMMY_PREAMBLE}\n\n## Capability Status\n- {chat_status}\n- {semantic_status}\n- {vision_status}\n\n## Current Context\n{context}"
        )
    }
}

fn initial_route_status(route: AgentRoute, has_images: bool) -> String {
    match route {
        AgentRoute::MealIntake => status_event(
            AgentStatusKind::ReadingInput,
            if has_images {
                "我在整理图片里的用餐线索。"
            } else {
                "我在整理这次用餐线索。"
            },
        ),
        AgentRoute::Chat if has_images => {
            status_event(AgentStatusKind::ReadingInput, "我在认真看看这张图片。")
        }
        AgentRoute::Chat => status_event(AgentStatusKind::Thinking, "我在准备这次对话的上下文。"),
    }
}

fn tool_choice_for_route(route: AgentRoute, _input: &str) -> ToolChoice {
    match route {
        AgentRoute::MealIntake => ToolChoice::Required,
        AgentRoute::Chat => ToolChoice::Auto,
    }
}

fn status_event(kind: AgentStatusKind, label: &str) -> String {
    stream_event(ConversationStreamEvent::Status {
        kind,
        label: label.to_string(),
    })
}

fn build_conversation_summary_prompt(previous_summary: &str, window: &[ChatMessage]) -> String {
    let previous_summary = if previous_summary.trim().is_empty() {
        "无".to_string()
    } else {
        previous_summary.trim().to_string()
    };
    let messages = window
        .iter()
        .enumerate()
        .map(|(index, message)| {
            format!(
                "{}. {}: {}",
                index + 1,
                message.role,
                message.content.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "previous summary:\n{previous_summary}\n\nnew message window:\n{messages}\n\n请输出合并后的 rolling conversation summary。"
    )
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
