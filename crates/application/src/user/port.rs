use async_trait::async_trait;
use domain::{UserId, UserProfile};

#[async_trait]
pub trait UserProfileRepositoryPort: Send + Sync {
    async fn find_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>, String>;
}
