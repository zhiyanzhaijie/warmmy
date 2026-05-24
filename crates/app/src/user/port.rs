use async_trait::async_trait;
use domain::{UserId, UserProfile};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant"
    pub content: String,
}

#[async_trait]
pub trait UserProfileRepositoryPort: Send + Sync {
    async fn find_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>, String>;
}

#[async_trait]
pub trait ChatMessageRepositoryPort: Send + Sync {
    async fn find_by_session(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<ChatMessage>, String>;
    async fn save_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), String>;
    async fn find_sessions(&self, user_id: &UserId) -> Result<Vec<String>, String>;
}
