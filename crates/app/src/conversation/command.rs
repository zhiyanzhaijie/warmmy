use std::pin::Pin;
use std::sync::Arc;

use domain::UserId;
use futures_core::Stream;

use crate::app_error::{AppError, AppResult};
use crate::conversation::ConversationAgentPort;

#[derive(Debug, Clone)]
pub struct SendUserMessageCommand {
    pub user_id: UserId,
    pub session_id: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SendUserMessageResult {
    pub reply: String,
    pub session_id: String,
}

pub type ConversationReplyStream = Pin<Box<dyn Stream<Item = Result<String, AppError>> + Send>>;

#[derive(Clone)]
pub struct ConversationCommandHandler {
    agent: Arc<dyn ConversationAgentPort>,
}

impl ConversationCommandHandler {
    pub fn new(agent: Arc<dyn ConversationAgentPort>) -> Self {
        Self { agent }
    }

    pub async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult> {
        self.agent.send_user_message(command).await
    }

    pub async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.agent.stream_user_message(command).await
    }
}
