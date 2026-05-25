use async_trait::async_trait;
use domain::{UserHealthExpectation, UserId, UserPreferences, UserProfile};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant"
    pub content: String,
}

#[async_trait]
pub trait UserProfileRepositoryPort: Send + Sync {
    async fn find_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>, String>;
    async fn save_profile(&self, profile: &UserProfile) -> Result<(), String>;
}

#[async_trait]
pub trait UserHealthExpectationRepositoryPort: Send + Sync {
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<UserHealthExpectation>, String>;

    async fn list_active_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<UserHealthExpectation>, String>;

    async fn save_expectation(&self, expectation: &UserHealthExpectation) -> Result<(), String>;

    async fn delete_expectation(
        &self,
        user_id: &UserId,
        expectation_id: &domain::HealthExpectationId,
    ) -> Result<(), String>;
}

#[async_trait]
pub trait UserPreferencesRepositoryPort: Send + Sync {
    async fn find_preferences(&self, user_id: &UserId) -> Result<Option<UserPreferences>, String>;

    async fn save_preferences(&self, preferences: &UserPreferences) -> Result<(), String>;
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
