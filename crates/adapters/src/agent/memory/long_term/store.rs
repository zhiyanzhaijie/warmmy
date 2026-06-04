use app::app_error::AppResult;
use async_trait::async_trait;

use domain::UserId;

use crate::agent::memory::types::{MemoryCandidate, MemoryRecord, MemoryStatus};

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn put(&self, candidate: MemoryCandidate) -> AppResult<MemoryRecord>;

    async fn get(&self, id: &str) -> AppResult<Option<MemoryRecord>>;

    async fn set_status(&self, id: &str, status: MemoryStatus) -> AppResult<Option<MemoryRecord>>;

    async fn list_by_status(
        &self,
        user_id: &UserId,
        status: MemoryStatus,
    ) -> AppResult<Vec<MemoryRecord>>;

    async fn list_confirmed_by_user(&self, user_id: &UserId) -> AppResult<Vec<MemoryRecord>>;
}
