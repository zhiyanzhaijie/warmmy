use std::sync::Arc;

use domain::UserId;

use crate::app_error::{AppError, AppResult};
use crate::conversation::ChatMessageRepositoryPort;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
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
