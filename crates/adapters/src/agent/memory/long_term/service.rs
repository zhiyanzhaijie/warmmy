use std::sync::Arc;

use app::app_error::AppResult;

use crate::agent::memory::long_term::index::MemoryIndex;
use crate::agent::memory::long_term::policy::MemoryPolicy;
use crate::agent::memory::long_term::promotion::{is_conflict, is_duplicate};
use crate::agent::memory::long_term::store::MemoryStore;
use domain::UserId;

use crate::agent::memory::types::{MemoryCandidate, MemoryRecord, MemoryStatus};

#[derive(Clone)]
pub struct MemoryService {
    policy: MemoryPolicy,
    store: Arc<dyn MemoryStore>,
    index: Arc<dyn MemoryIndex>,
}

impl MemoryService {
    pub fn new(
        policy: MemoryPolicy,
        store: Arc<dyn MemoryStore>,
        index: Arc<dyn MemoryIndex>,
    ) -> Self {
        Self {
            policy,
            store,
            index,
        }
    }

    pub async fn commit(&self, candidate: MemoryCandidate) -> AppResult<Option<MemoryRecord>> {
        if !self.policy.accepts(&candidate) {
            tracing::info!(
                memory.form = ?candidate.form,
                memory.scope = ?candidate.scope,
                memory.status = ?candidate.status,
                memory.index_policy = ?candidate.index_policy,
                "memory candidate rejected by policy"
            );
            return Ok(None);
        }

        let record = self.store.put(candidate).await?;
        tracing::info!(
            memory.id = record.id.as_str(),
            memory.form = ?record.form,
            memory.scope = ?record.scope,
            memory.status = ?record.status,
            memory.index_policy = ?record.index_policy,
            "memory committed"
        );
        if self.policy.indexes(&record) {
            self.index.put(&record).await?;
            tracing::info!(
                memory.id = record.id.as_str(),
                "memory indexed after commit"
            );
        }
        Ok(Some(record))
    }

    pub async fn commit_all(
        &self,
        candidates: impl IntoIterator<Item = MemoryCandidate>,
    ) -> AppResult<()> {
        for candidate in candidates {
            self.commit(candidate).await?;
        }
        Ok(())
    }

    pub async fn promote(&self, id: &str) -> AppResult<Option<MemoryRecord>> {
        let Some(record) = self.store.set_status(id, MemoryStatus::Confirmed).await? else {
            return Ok(None);
        };
        tracing::info!(
            memory.id = record.id.as_str(),
            memory.form = ?record.form,
            memory.scope = ?record.scope,
            memory.index_policy = ?record.index_policy,
            "memory promoted"
        );
        for existing in self.store.list_confirmed_by_user(&record.user_id).await? {
            if existing.id != record.id && is_duplicate(&existing, &record) {
                tracing::info!(
                    memory.id = record.id.as_str(),
                    duplicate.id = existing.id.as_str(),
                    "memory rejected as duplicate"
                );
                return self
                    .store
                    .set_status(&record.id, MemoryStatus::Rejected)
                    .await;
            }
            if existing.id != record.id && is_conflict(&existing, &record) {
                tracing::info!(
                    memory.id = record.id.as_str(),
                    superseded.id = existing.id.as_str(),
                    "memory superseded conflicting record"
                );
                self.store
                    .set_status(&existing.id, MemoryStatus::Superseded)
                    .await?;
            }
        }
        if self.policy.indexes(&record) {
            self.index.put(&record).await?;
            tracing::info!(
                memory.id = record.id.as_str(),
                "memory indexed after promotion"
            );
        }
        Ok(Some(record))
    }

    pub async fn dismiss(&self, id: &str) -> AppResult<Option<MemoryRecord>> {
        self.store.set_status(id, MemoryStatus::Rejected).await
    }

    pub async fn expire(&self, id: &str) -> AppResult<Option<MemoryRecord>> {
        self.store.set_status(id, MemoryStatus::Expired).await
    }

    pub async fn list_candidates(&self, user_id: &UserId) -> AppResult<Vec<MemoryRecord>> {
        self.store
            .list_by_status(user_id, MemoryStatus::Candidate)
            .await
    }

    pub async fn list_confirmed(&self, user_id: &UserId) -> AppResult<Vec<MemoryRecord>> {
        self.store
            .list_by_status(user_id, MemoryStatus::Confirmed)
            .await
    }
}
