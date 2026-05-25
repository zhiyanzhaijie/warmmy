use std::sync::Arc;

use app::app_error::AppResult;
use app::conversation::{
    ChatMessageRepositoryPort, ConversationAgentPort, ConversationReplyStream,
    SendUserMessageCommand, SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::UserDietaryContextQueryHandler;
use async_trait::async_trait;

use crate::agent::config::AgentConfig;
use crate::agent::runtime::rig::RigConversationRuntime;

pub struct ConversationAgentService {
    runtime: RigConversationRuntime,
}

impl ConversationAgentService {
    pub fn new(
        config: AgentConfig,
        meal_command: Arc<MealCommandHandler>,
        repo: Arc<dyn ChatMessageRepositoryPort>,
        user_contexts: UserDietaryContextQueryHandler,
    ) -> Self {
        let runtime = RigConversationRuntime::new(config, meal_command, repo, user_contexts);
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
            .complete(&command.user_id, &command.session_id, &command.content)
            .await
    }

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.runtime
            .stream(&command.user_id, &command.session_id, command.content)
            .await
    }
}
