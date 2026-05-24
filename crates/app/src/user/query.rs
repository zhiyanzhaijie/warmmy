use std::sync::Arc;

use domain::{UserId, UserProfile};

use crate::app_error::{AppError, AppResult};
use crate::user::{ChatMessage, ChatMessageRepositoryPort, UserProfileRepositoryPort};

#[derive(Clone)]
pub struct UserProfileQueryHandler {
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl UserProfileQueryHandler {
    pub fn new(user_profiles: Arc<dyn UserProfileRepositoryPort>) -> Self {
        Self { user_profiles }
    }

    pub async fn get_profile(&self, user_id: &UserId) -> AppResult<Option<UserProfile>> {
        self.user_profiles
            .find_profile(user_id)
            .await
            .map_err(AppError::upstream)
    }
}

#[derive(Clone)]
pub struct UserChatQueryHandler {
    repo: Arc<dyn ChatMessageRepositoryPort>,
}

impl UserChatQueryHandler {
    pub fn new(repo: Arc<dyn ChatMessageRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get_session_history(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> AppResult<Vec<ChatMessage>> {
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
