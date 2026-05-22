use std::sync::Arc;
use async_trait::async_trait;
use domain::UserId;
use futures_core::Stream;
use std::pin::Pin;

use crate::app_error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant"
    pub content: String,
}

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

#[async_trait]
pub trait SendUserMessageUseCase: Send + Sync {
    async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult>;

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream>;
}

#[async_trait]
pub trait ChatMessageRepositoryPort: Send + Sync {
    async fn find_by_session(&self, user_id: &UserId, session_id: &str) -> Result<Vec<ChatMessage>, String>;
    async fn save_message(&self, user_id: &UserId, session_id: &str, role: &str, content: &str) -> Result<(), String>;
    async fn find_sessions(&self, user_id: &UserId) -> Result<Vec<String>, String>;
}

#[derive(Clone)]
pub struct ConversationQueryHandler {
    repo: Arc<dyn ChatMessageRepositoryPort>,
}

impl ConversationQueryHandler {
    pub fn new(repo: Arc<dyn ChatMessageRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get_session_history(&self, user_id: &UserId, session_id: &str) -> AppResult<Vec<ChatMessage>> {
        self.repo
            .find_by_session(user_id, session_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn list_user_sessions(&self, user_id: &UserId) -> AppResult<Vec<String>> {
        self.repo
            .find_sessions(user_id)
            .await
            .map_err(AppError::upstream)
    }
}
