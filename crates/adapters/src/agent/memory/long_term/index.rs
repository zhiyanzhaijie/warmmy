use app::app_error::AppResult;
use async_trait::async_trait;

use crate::agent::memory::types::MemoryRecord;

#[async_trait]
pub trait MemoryIndex: Send + Sync {
    async fn put(&self, record: &MemoryRecord) -> AppResult<()>;
}

#[derive(Clone, Default)]
pub struct NoopMemoryIndex;

#[async_trait]
impl MemoryIndex for NoopMemoryIndex {
    async fn put(&self, _record: &MemoryRecord) -> AppResult<()> {
        Ok(())
    }
}
