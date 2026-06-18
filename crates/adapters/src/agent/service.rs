use std::sync::Arc;

use app::agents::{prompts::interaction::interaction_continuation_prompt, MemoryStore};
use app::app_error::AppResult;
use app::conversation::{
    AgentStatusKind, ChatMessageRepositoryPort, ContinueInteractionCommand, ConversationAgentPort,
    ConversationReplyStream, ConversationStreamEvent, ConversationUserInput,
    EphemeralImageStorePort, SendUserMessageCommand, SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::{UserAIConfigQueryHandler, UserDietaryContextQueryHandler};
use async_trait::async_trait;
use crate::agent::runtime::events::{status_event, stream_event};

use crate::agent::runtime::rig::RigConversationRuntime;

pub struct ConversationAgentService {
    runtime: RigConversationRuntime,
}

impl ConversationAgentService {
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
        let runtime = RigConversationRuntime::new(
            meal_command,
            repo,
            image_store,
            memory_store,
            user_contexts,
            ai_configs,
            lancedb_path,
            rag_top_k,
        );
        Self { runtime }
    }
}

#[async_trait]
impl ConversationAgentPort for ConversationAgentService {
    async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult> {
        self.runtime
            .complete(&command.user_id, &command.session_id, command.input)
            .await
    }

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.runtime
            .stream(&command.user_id, &command.session_id, command.input)
            .await
    }

    async fn continue_interaction(
        &self,
        command: ContinueInteractionCommand,
    ) -> AppResult<ConversationReplyStream> {
        let continuation_prompt = interaction_continuation_prompt(command.interaction.clone());
        let is_pending_continuation = matches!(
            command.interaction,
            app::conversation::AgentInteractionContinuation::ConfirmMealLog { .. }
                | app::conversation::AgentInteractionContinuation::RejectMealLog { .. }
        );
        if is_pending_continuation {
            let result = self
                .runtime
                .complete(
                    &command.user_id,
                    &command.session_id,
                    ConversationUserInput::text_only(continuation_prompt),
                )
                .await?;
            let reply = result.reply;
            let s = async_stream::stream! {
                yield Ok(status_event(AgentStatusKind::CallingModel, "我在执行这次确认操作。"));
                if !reply.trim().is_empty() {
                    yield Ok(stream_event(ConversationStreamEvent::TextDelta { text: reply }));
                }
                yield Ok(stream_event(ConversationStreamEvent::Done));
            };
            return Ok(Box::pin(s));
        }

        self.runtime
            .stream(
                &command.user_id,
                &command.session_id,
                ConversationUserInput::text_only(continuation_prompt),
            )
            .await
    }
}
