use std::sync::Arc;

use app::app_error::AppError;
use chrono::{DateTime, Utc};
use rig::vector_store::{VectorSearchRequest, VectorStoreError, VectorStoreIndex};
use serde::Deserialize;
use serde_json::json;

use crate::agent::memory::long_term::rag::{LanceDbFilter, OpenAiCompatibleRagIndex};
use crate::agent::memory::long_term::store::MemoryStore;
use crate::agent::memory::types::{MemoryRecord, MemoryScope, MemoryStatus};
use domain::UserId;

pub struct MemoryRetriever {
    user_id: UserId,
    store: Arc<dyn MemoryStore>,
    index: OpenAiCompatibleRagIndex,
}

impl MemoryRetriever {
    pub fn new(
        user_id: UserId,
        store: Arc<dyn MemoryStore>,
        index: OpenAiCompatibleRagIndex,
    ) -> Self {
        Self {
            user_id,
            store,
            index,
        }
    }
}

impl VectorStoreIndex for MemoryRetriever {
    type Filter = LanceDbFilter;

    async fn top_n<T: for<'a> Deserialize<'a> + Send>(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
        let ids = self.index.top_n_ids(req).await?;
        tracing::info!(
            user.id = self.user_id.as_str(),
            memory.hit_count = ids.len(),
            "memory retriever vector hits"
        );
        let mut results = Vec::new();
        let mut missing = 0usize;
        let mut filtered = 0usize;

        for (score, id) in ids {
            let Some(record) = self
                .store
                .get(&id)
                .await
                .map_err(app_error_to_vector_store_error)?
            else {
                missing += 1;
                continue;
            };

            if !self.can_retrieve(&record) {
                filtered += 1;
                continue;
            }

            let content = retrieval_content(&record);
            let value = json!({
                "id": record.id,
                "content": content,
                "raw_content": record.content,
                "scope": scope_metadata(&record.scope),
                "created_at": record.created_at,
                "updated_at": record.updated_at,
                "source": "memory_record",
                "score": score,
            });
            let document = serde_json::from_value(value)?;
            results.push((score, id, document));
        }

        tracing::info!(
            user.id = self.user_id.as_str(),
            memory.retained_count = results.len(),
            memory.missing_count = missing,
            memory.filtered_count = filtered,
            "memory retriever hydrated hits"
        );
        Ok(results)
    }

    async fn top_n_ids(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String)>, VectorStoreError> {
        Ok(self
            .top_n::<serde_json::Value>(req)
            .await?
            .into_iter()
            .map(|(score, id, _)| (score, id))
            .collect())
    }
}

impl MemoryRetriever {
    fn can_retrieve(&self, record: &MemoryRecord) -> bool {
        record.user_id == self.user_id
            && record.status == MemoryStatus::Confirmed
            && !is_expired(&record.scope)
    }
}

fn is_expired(scope: &MemoryScope) -> bool {
    let MemoryScope::Temporary { expires_at } = scope else {
        return false;
    };

    DateTime::parse_from_rfc3339(expires_at)
        .map(|expires_at| expires_at.with_timezone(&Utc) <= Utc::now())
        .unwrap_or(false)
}

fn retrieval_content(record: &MemoryRecord) -> String {
    match &record.scope {
        MemoryScope::Day { date } => format!("日期 {date}：{}", record.content),
        MemoryScope::UserGlobal => record.content.clone(),
        MemoryScope::Temporary { expires_at } => {
            format!("临时记忆，有效至 {expires_at}：{}", record.content)
        }
    }
}

fn scope_metadata(scope: &MemoryScope) -> serde_json::Value {
    match scope {
        MemoryScope::Day { date } => json!({
            "kind": "day",
            "date": date,
        }),
        MemoryScope::UserGlobal => json!({
            "kind": "user_global",
        }),
        MemoryScope::Temporary { expires_at } => json!({
            "kind": "temporary",
            "expires_at": expires_at,
        }),
    }
}

fn app_error_to_vector_store_error(err: AppError) -> VectorStoreError {
    VectorStoreError::DatastoreError(Box::new(err))
}
