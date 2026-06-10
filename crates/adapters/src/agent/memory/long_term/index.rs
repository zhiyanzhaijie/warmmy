use app::agents::{MemoryIndex, MemoryRecord};
use app::app_error::AppResult;
use async_trait::async_trait;

#[derive(Clone, Default)]
pub struct NoopMemoryIndex;

#[async_trait]
impl MemoryIndex for NoopMemoryIndex {
    async fn put(&self, _record: &MemoryRecord) -> AppResult<()> {
        Ok(())
    }
}
