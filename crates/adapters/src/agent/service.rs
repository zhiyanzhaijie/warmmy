use std::sync::Arc;

use app::agents::{prompts::interaction::interaction_continuation_prompt, MemoryStore};
use app::app_error::AppResult;
use app::conversation::{
    ChatMessageRepositoryPort, ContinueInteractionCommand, ConversationAgentPort,
    ConversationReplyStream, ConversationUserInput, EphemeralImageStorePort,
    SendUserMessageCommand, SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::{UserAIConfigQueryHandler, UserDietaryContextQueryHandler};
use async_trait::async_trait;

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
        self.runtime
            .stream(
                &command.user_id,
                &command.session_id,
                ConversationUserInput::text_only(interaction_continuation_prompt(
                    command.interaction,
                )),
            )
            .await
    }
}
