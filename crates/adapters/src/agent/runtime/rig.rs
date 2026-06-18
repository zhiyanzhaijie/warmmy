use std::sync::Arc;

use app::agents::{
    prompts::conversation_summary::CONVERSATION_SUMMARY_PREAMBLE, AgentRoute, AgentServiceProgress,
    AgentToolId, AgentTurnInput, AgentTurnPlan, AgentTurnPlanner, DefaultAgentTurnPlanner,
    MemoryIndex, MemoryObservation, MemoryPolicy, MemoryStore, NutritionCurator,
    NutritionReferenceRetriever, RoutePlanner, RouteRegistry, RuleThenModelRoutePlanner,
    TextModelGateway, TextPromptRequest,
};
use app::app_error::{AppError, AppResult};
use app::conversation::{
    AgentStatusKind, ChatMessage, ChatMessageRepositoryPort, ConversationReplyStream,
    ConversationStreamEvent, ConversationSummary, ConversationUserInput, EphemeralImageStorePort,
    SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::{ResolvedAIModelConfig, UserAIConfigQueryHandler, UserDietaryContextQueryHandler};
use async_stream::stream;
use futures_util::StreamExt;
use rig::agent::{MultiTurnStreamItem, StreamingError};
use rig::message::ToolChoice;
use rig::streaming::StreamedAssistantContent;
use tokio::sync::mpsc;

use crate::agent::interaction::AgentInteractionSink;
use crate::agent::memory::long_term::extractor::ModelMemoryExtractor;
use crate::agent::memory::long_term::index::NoopMemoryIndex;
use crate::agent::memory::long_term::pipeline::MemoryPipeline;
use crate::agent::memory::long_term::rag::{build_rag_index, LanceDbMemoryIndex, RagConfig};
use crate::agent::memory::long_term::retriever::MemoryRetriever;
use crate::agent::memory::long_term::service::MemoryService;
use crate::agent::memory::MemoryContextProvider;
use crate::agent::memory::SessionConversationMemory;
use crate::agent::model::{AgentSpec, ModelGateway, RagAgentContext, RigModelFactory};
use crate::agent::runtime::events::{
    initial_route_status, interaction_event, status_event, stream_event,
};
use crate::agent::runtime::hook::{AgentStatusSink, GuardrailHook};
use crate::agent::runtime::image::{build_prompt_message, persist_user_image_message};
use crate::agent::runtime::persistence::{
    is_internal_conversation_input, persist_assistant_visible_message, persist_user_visible_message,
};
use crate::agent::services::nutrition::curator::ModelNutritionCurator;
use crate::agent::services::nutrition::retriever::LanceDbNutritionReferenceRetriever;
use crate::agent::tool;
use domain::{AICapability, UserId};

const DEFAULT_HISTORY_WINDOW_MESSAGES: usize = 16;
const DEFAULT_EMBEDDING_NDIMS: usize = 2048;
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

pub struct StreamWrapCtx {
    interaction_sink: AgentInteractionSink,
    has_images: bool,
    repo_for_history: Arc<dyn ChatMessageRepositoryPort>,
    user_id_for_history: UserId,
    session_id_for_history: String,
    runtime_for_memory: RigConversationRuntime,
    user_id_for_memory: UserId,
    session_id_for_memory: String,
    user_input_for_memory: String,
    should_persist_assistant_visible_message: bool,
    memory_observation_enabled: bool,
    should_update_conversation_summary: bool,
    status_rx: mpsc::UnboundedReceiver<ConversationStreamEvent>,
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
    model_gateway: Arc<dyn ModelGateway>,
    text_model: Arc<dyn TextModelGateway>,
    turn_planner: Arc<dyn AgentTurnPlanner>,
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
        let model_factory = Arc::new(RigModelFactory::new());
        let model_gateway: Arc<dyn ModelGateway> = model_factory.clone();
        let text_model: Arc<dyn TextModelGateway> = model_factory.clone();
        let route_registry = RouteRegistry::new();
        let route_planner: Arc<dyn RoutePlanner> = Arc::new(RuleThenModelRoutePlanner::new(
            route_registry.clone(),
            text_model.clone(),
        ));
        let turn_planner: Arc<dyn AgentTurnPlanner> =
            Arc::new(DefaultAgentTurnPlanner::new(route_planner, route_registry));
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
            model_gateway,
            text_model,
            turn_planner,
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
        Some(Arc::new(ModelNutritionCurator::with_model_gateway(
            chat.clone(),
            self.text_model.clone(),
        )))
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
        tool_ids: &[AgentToolId],
        tool_choice: ToolChoice,
        interaction_sink: AgentInteractionSink,
        nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
        rag: Option<RagAgentContext>,
        status_sink: Option<AgentStatusSink>,
    ) -> AgentSpec {
        let tools = tool::tools_for_ids(
            tool_ids,
            user_id,
            session_id,
            self.meal_command.clone(),
            interaction_sink,
            self.nutrition_curator(chat),
            nutrition_retriever,
            self.agent_service_progress(status_sink.clone()),
        );
        tracing::info!(
            agent.route = ?route,
            tool.count = tools.len(),
            "agent tools selected"
        );

        AgentSpec {
            preamble,
            tool_choice,
            memory: self.build_memory(user_id),
            tools,
            rag,
            guardrail: self.guardrail.clone(),
            status_sink,
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

    async fn plan_turn(
        &self,
        user_id: &UserId,
        session_id: &str,
        model: ResolvedAIModelConfig,
        text: String,
        has_images: bool,
        rag_enabled: bool,
    ) -> AppResult<AgentTurnPlan> {
        let context = self.context_provider.load(user_id).await;
        let is_internal_input = is_internal_conversation_input(&text);
        let plan = self
            .turn_planner
            .plan_turn(AgentTurnInput {
                text,
                has_images,
                is_internal_input,
                model,
                current_context: context,
                chat_enabled: true,
                semantic_memory_enabled: rag_enabled,
                memory_observation_enabled: true,
            })
            .await?;
        tracing::info!(
            agent.route = ?plan.route(),
            route.source = ?plan.route_decision.source,
            route.reason = %plan.route_decision.reason,
            session.id = %session_id,
            input.has_images = has_images,
            "agent turn planned"
        );
        Ok(plan)
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
        let interaction_sink = AgentInteractionSink::default();
        let memory_input = input.visible_text();
        let is_internal_input = is_internal_conversation_input(&memory_input);
        let plan = self
            .plan_turn(
                user_id,
                session_id,
                chat.clone(),
                memory_input.clone(),
                input.has_images(),
                rag.is_some(),
            )
            .await?;
        let route = plan.route();
        let tool_choice = tool_choice_for_route(route, &memory_input);
        let prompt = build_prompt_message(&self.image_store, &input).await?;
        if plan.lifecycle.persist_user_image_message {
            persist_user_image_message(&self.repo, user_id, session_id, &input, &memory_input)
                .await;
        }
        if plan.lifecycle.persist_user_visible_message {
            persist_user_visible_message(&self.repo, user_id, session_id, &input, &memory_input)
                .await?;
        }
        let rag_context = if is_internal_input {
            None
        } else {
            self.build_rag_agent_context(user_id, rag.as_ref()).await?
        };
        let tool_ids = plan.tool_ids.clone();
        let spec = self.build_agent_spec(
            user_id,
            session_id,
            &chat,
            plan.preamble.clone(),
            route,
            &tool_ids,
            tool_choice,
            interaction_sink,
            nutrition_retriever,
            rag_context,
            None,
        );
        let reply = self
            .model_gateway
            .conversation_agent(&chat, spec)?
            .prompt(prompt, session_id)
            .await?;

        if plan.lifecycle.persist_assistant_visible_message {
            persist_assistant_visible_message(&self.repo, user_id, session_id, &reply).await?;
        }
        if plan.lifecycle.memory_observation_enabled {
            self.observe_memory(user_id, session_id, &memory_input, &reply)
                .await;
        }
        if plan.lifecycle.update_conversation_summary {
            self.spawn_conversation_summary_update(user_id, session_id);
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
        input: ConversationUserInput,
    ) -> AppResult<ConversationReplyStream> {
        let chat = self.resolve_conversation_model(user_id, &input).await?;
        let rag = self.resolve_rag(user_id).await?;
        let runtime = self.clone();
        let user_id = user_id.clone();
        let session_id = session_id.to_string();
        let has_images = input.has_images();
        let visible_text = input.visible_text();
        let plan = self
            .plan_turn(
                &user_id,
                &session_id,
                chat.clone(),
                visible_text,
                has_images,
                rag.is_some(),
            )
            .await?;
        let route = plan.route();

        Ok(Box::pin(stream! {
            yield Ok(stream_event(ConversationStreamEvent::RunStarted {
                run_id: format!("{user_id}:{session_id}"),
            }));
            yield Ok(initial_route_status(route, has_images));

            match runtime
                .open_stream_with_model(&user_id, &session_id, input, chat, plan, rag)
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
        plan: AgentTurnPlan,
        rag: Option<RagConfig>,
    ) -> AppResult<ConversationReplyStream> {
        let nutrition_retriever = self.nutrition_retriever(rag.as_ref());
        let interaction_sink = AgentInteractionSink::default();
        let user_input = input.visible_text();
        let is_internal_input = is_internal_conversation_input(&user_input);
        let has_images = input.has_images();
        let route = plan.route();
        let tool_ids = plan.tool_ids.clone();
        let tool_choice = tool_choice_for_route(route, &user_input);
        let prompt = build_prompt_message(&self.image_store, &input).await?;
        if plan.lifecycle.persist_user_image_message {
            persist_user_image_message(&self.repo, user_id, session_id, &input, &user_input).await;
        }
        if plan.lifecycle.persist_user_visible_message {
            persist_user_visible_message(&self.repo, user_id, session_id, &input, &user_input)
                .await?;
        }
        let user_id_for_history = user_id.clone();
        let user_id_for_memory = user_id.clone();
        let session_id_for_history = session_id.to_string();
        let session_id_for_memory = session_id.to_string();
        let repo_for_history = self.repo.clone();
        let runtime_for_memory = self.clone();
        let user_input_for_memory = user_input.clone();
        let (status_tx, status_rx) = mpsc::unbounded_channel();
        let status_sink = AgentStatusSink::new(status_tx);
        let rag_context = if is_internal_input {
            None
        } else {
            self.build_rag_agent_context(user_id, rag.as_ref()).await?
        };
        let spec = self.build_agent_spec(
            user_id,
            session_id,
            &chat,
            plan.preamble.clone(),
            route,
            &tool_ids,
            tool_choice,
            interaction_sink.clone(),
            nutrition_retriever,
            rag_context,
            Some(status_sink.clone()),
        );
        let agent = self.model_gateway.conversation_agent(&chat, spec)?;
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
            should_persist_assistant_visible_message: plan
                .lifecycle
                .persist_assistant_visible_message,
            memory_observation_enabled: plan.lifecycle.memory_observation_enabled,
            should_update_conversation_summary: plan.lifecycle.update_conversation_summary,
            status_rx,
        };
        let stream = agent.stream(prompt, session_id.to_string(), wrap_ctx).await?;
        Ok(stream)
    }

    pub(crate) fn wrap_stream<S, R>(mut raw: S, ctx: StreamWrapCtx) -> ConversationReplyStream
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
            should_persist_assistant_visible_message,
            memory_observation_enabled,
            should_update_conversation_summary,
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
                            assistant_output.push_str(&text.text);
                            yield Ok(stream_event(ConversationStreamEvent::TextDelta {
                                text: text.text,
                            }));
                        }
                    }
                    Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                        if !has_text_delta && !r.response().is_empty() {
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
                        if should_persist_assistant_visible_message {
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
                            if memory_observation_enabled {
                                runtime_for_memory
                                    .observe_memory(
                                        &user_id_for_memory,
                                        &session_id_for_memory,
                                        &user_input_for_memory,
                                        &assistant_output_for_memory,
                                    )
                                    .await;
                            }
                            if should_update_conversation_summary {
                                runtime_for_memory.spawn_conversation_summary_update(
                                    &user_id_for_memory,
                                    &session_id_for_memory,
                                );
                            }
                        });
                        yield Ok(stream_event(ConversationStreamEvent::Done));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        yield Err(streaming_upstream_error(&e));
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
            embedding_provider: embedding.provider,
            embedding_base_url: embedding.base_url,
            embedding_api_key: embedding.api_key,
            embedding_model: embedding.model,
            embedding_ndims: DEFAULT_EMBEDDING_NDIMS,
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

        let memory_index: Arc<dyn MemoryIndex> = match self.resolve_rag(user_id).await {
            Ok(Some(rag)) => Arc::new(LanceDbMemoryIndex::new(rag)),
            Ok(None) => Arc::new(NoopMemoryIndex),
            Err(err) => {
                tracing::warn!(error = %err, "failed to resolve memory index");
                Arc::new(NoopMemoryIndex)
            }
        };

        let pipeline = MemoryPipeline::new(
            Arc::new(ModelMemoryExtractor::with_model_gateway(
                extractor_model,
                self.text_model.clone(),
            )),
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
        self.text_model
            .prompt_text(TextPromptRequest {
                model: model.clone(),
                preamble: CONVERSATION_SUMMARY_PREAMBLE.to_string(),
                prompt,
            })
            .await
    }
}

fn tool_choice_for_route(route: AgentRoute, _input: &str) -> ToolChoice {
    match route {
        AgentRoute::MealIntake => ToolChoice::Auto,
        AgentRoute::Chat => ToolChoice::Auto,
    }
}


fn streaming_upstream_error(error: &impl std::fmt::Display) -> AppError {
    let raw = error.to_string();
    if raw.contains("data_inspection_failed") || raw.contains("DataInspectionFailed") {
        tracing::warn!(
            error = %raw,
            "model provider rejected streamed output during content inspection"
        );
        return AppError::upstream("模型服务内容安全检查未通过，无法返回这次图片识别结果");
    }

    AppError::upstream(raw)
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
