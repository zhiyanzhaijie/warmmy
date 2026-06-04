use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::agent::memory::long_term::store::MemoryStore;
use crate::agent::memory::types::{
    IndexPolicy, LongTermMemoryForm, MemoryCandidate, MemoryClaim, MemoryRecord, MemoryScope,
    MemorySource, MemoryStatus,
};
use crate::persistence::sqlite::models::MemoryRecordRow;

#[derive(Clone)]
pub struct SqliteMemoryStore {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteMemoryStore {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn put(&self, candidate: MemoryCandidate) -> AppResult<MemoryRecord> {
        let now = Utc::now().to_rfc3339();
        let record = MemoryRecord {
            id: new_memory_id(),
            user_id: candidate.user_id,
            form: candidate.form,
            scope: candidate.scope,
            source: candidate.source,
            status: candidate.status,
            index_policy: candidate.index_policy,
            content: candidate.content,
            search_text: candidate.search_text,
            confidence: candidate.confidence,
            claim: candidate.claim,
            created_at: now.clone(),
            updated_at: now,
        };

        let row = row_from_record(&record)?;
        let mut db = self.db.lock().await;
        toasty::create!(MemoryRecordRow {
            id: row.id,
            user_id: row.user_id,
            tier: "long_term".to_string(),
            form: row.form,
            scope_json: row.scope_json,
            source_json: row.source_json,
            status: row.status,
            index_policy: row.index_policy,
            content: row.content,
            search_text: row.search_text,
            confidence: row.confidence,
            claim_json: row.claim_json,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .exec(&mut *db)
        .await
        .map_err(|err| AppError::database(err.to_string()))?;

        Ok(record)
    }

    async fn get(&self, id: &str) -> AppResult<Option<MemoryRecord>> {
        let mut db = self.db.lock().await;
        let rows = MemoryRecordRow::filter(MemoryRecordRow::fields().id().eq(id))
            .exec(&mut *db)
            .await
            .map_err(|err| AppError::database(err.to_string()))?;

        rows.into_iter().next().map(record_from_row).transpose()
    }

    async fn set_status(&self, id: &str, status: MemoryStatus) -> AppResult<Option<MemoryRecord>> {
        let mut db = self.db.lock().await;
        let mut row = match MemoryRecordRow::get_by_id(&mut *db, &id.to_string()).await {
            Ok(row) => row,
            Err(err) if err.is_record_not_found() => return Ok(None),
            Err(err) => return Err(AppError::database(err.to_string())),
        };

        let updated_at = Utc::now().to_rfc3339();
        row.update()
            .status(status_to_str(&status).to_string())
            .updated_at(updated_at)
            .exec(&mut *db)
            .await
            .map_err(|err| AppError::database(err.to_string()))?;

        let rows = MemoryRecordRow::filter(MemoryRecordRow::fields().id().eq(id))
            .exec(&mut *db)
            .await
            .map_err(|err| AppError::database(err.to_string()))?;

        rows.into_iter().next().map(record_from_row).transpose()
    }

    async fn list_by_status(
        &self,
        user_id: &domain::UserId,
        status: MemoryStatus,
    ) -> AppResult<Vec<MemoryRecord>> {
        let mut db = self.db.lock().await;
        let rows = MemoryRecordRow::filter(
            MemoryRecordRow::fields()
                .user_id()
                .eq(user_id.as_str())
                .and(
                    MemoryRecordRow::fields()
                        .status()
                        .eq(status_to_str(&status)),
                ),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| AppError::database(err.to_string()))?;

        rows.into_iter().map(record_from_row).collect()
    }

    async fn list_confirmed_by_user(
        &self,
        user_id: &domain::UserId,
    ) -> AppResult<Vec<MemoryRecord>> {
        self.list_by_status(user_id, MemoryStatus::Confirmed).await
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum StoredScope {
    Day { date: String },
    UserGlobal,
    Temporary { expires_at: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum StoredSource {
    UserUtterance { session_id: String },
    AgentReflection { run_id: String },
    DomainEvent { entity: String, id: String },
}

struct StoredRecord {
    id: String,
    user_id: String,
    form: String,
    scope_json: String,
    source_json: String,
    status: String,
    index_policy: String,
    content: String,
    search_text: String,
    confidence: f32,
    claim_json: Option<String>,
    created_at: String,
    updated_at: String,
}

fn row_from_record(record: &MemoryRecord) -> AppResult<StoredRecord> {
    Ok(StoredRecord {
        id: record.id.clone(),
        user_id: record.user_id.as_str().to_string(),
        form: form_to_str(&record.form).to_string(),
        scope_json: serde_json::to_string(&scope_to_stored(&record.scope))
            .map_err(|err| AppError::internal(err.to_string()))?,
        source_json: serde_json::to_string(&source_to_stored(&record.source))
            .map_err(|err| AppError::internal(err.to_string()))?,
        status: status_to_str(&record.status).to_string(),
        index_policy: index_policy_to_str(&record.index_policy).to_string(),
        content: record.content.clone(),
        search_text: record.search_text.clone(),
        confidence: record.confidence,
        claim_json: record
            .claim
            .as_ref()
            .map(|claim| serde_json::to_string(claim))
            .transpose()
            .map_err(|err| AppError::internal(err.to_string()))?,
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    })
}

fn record_from_row(row: MemoryRecordRow) -> AppResult<MemoryRecord> {
    let scope: StoredScope = serde_json::from_str(&row.scope_json)
        .map_err(|err| AppError::database(err.to_string()))?;
    let source: StoredSource = serde_json::from_str(&row.source_json)
        .map_err(|err| AppError::database(err.to_string()))?;
    Ok(MemoryRecord {
        id: row.id,
        user_id: domain::UserId::new_unchecked(row.user_id),
        form: row.form.as_deref().map(form_from_str).transpose()?.ok_or_else(|| {
            AppError::database("long-term memory record missing form".to_string())
        })?,
        scope: scope_from_stored(scope),
        source: source_from_stored(source),
        status: status_from_str(&row.status)?,
        index_policy: index_policy_from_str(&row.index_policy)?,
        content: row.content,
        search_text: row.search_text,
        confidence: row.confidence,
        claim: row
            .claim_json
            .as_deref()
            .map(serde_json::from_str::<MemoryClaim>)
            .transpose()
            .map_err(|err| AppError::database(err.to_string()))?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn new_memory_id() -> String {
    format!("mem_{}", Utc::now().timestamp_nanos_opt().unwrap_or_default())
}

fn scope_to_stored(scope: &MemoryScope) -> StoredScope {
    match scope {
        MemoryScope::Day { date } => StoredScope::Day { date: date.clone() },
        MemoryScope::UserGlobal => StoredScope::UserGlobal,
        MemoryScope::Temporary { expires_at } => StoredScope::Temporary {
            expires_at: expires_at.clone(),
        },
    }
}

fn scope_from_stored(scope: StoredScope) -> MemoryScope {
    match scope {
        StoredScope::Day { date } => MemoryScope::Day { date },
        StoredScope::UserGlobal => MemoryScope::UserGlobal,
        StoredScope::Temporary { expires_at } => MemoryScope::Temporary { expires_at },
    }
}

fn source_to_stored(source: &MemorySource) -> StoredSource {
    match source {
        MemorySource::UserUtterance { session_id } => StoredSource::UserUtterance {
            session_id: session_id.clone(),
        },
        MemorySource::AgentReflection { run_id } => StoredSource::AgentReflection {
            run_id: run_id.clone(),
        },
        MemorySource::DomainEvent { entity, id } => StoredSource::DomainEvent {
            entity: entity.clone(),
            id: id.clone(),
        },
    }
}

fn source_from_stored(source: StoredSource) -> MemorySource {
    match source {
        StoredSource::UserUtterance { session_id } => MemorySource::UserUtterance { session_id },
        StoredSource::AgentReflection { run_id } => MemorySource::AgentReflection { run_id },
        StoredSource::DomainEvent { entity, id } => MemorySource::DomainEvent { entity, id },
    }
}

fn form_to_str(form: &LongTermMemoryForm) -> &'static str {
    match form {
        LongTermMemoryForm::Semantic => "semantic",
        LongTermMemoryForm::Episodic => "episodic",
        LongTermMemoryForm::Procedural => "procedural",
    }
}

fn form_from_str(value: &str) -> AppResult<LongTermMemoryForm> {
    match value {
        "semantic" => Ok(LongTermMemoryForm::Semantic),
        "episodic" => Ok(LongTermMemoryForm::Episodic),
        "procedural" => Ok(LongTermMemoryForm::Procedural),
        _ => Err(AppError::database(format!("unknown memory form: {value}"))),
    }
}

fn status_to_str(status: &MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Candidate => "candidate",
        MemoryStatus::Confirmed => "confirmed",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Rejected => "rejected",
        MemoryStatus::Expired => "expired",
    }
}

fn status_from_str(value: &str) -> AppResult<MemoryStatus> {
    match value {
        "candidate" => Ok(MemoryStatus::Candidate),
        "confirmed" => Ok(MemoryStatus::Confirmed),
        "superseded" => Ok(MemoryStatus::Superseded),
        "rejected" => Ok(MemoryStatus::Rejected),
        "expired" => Ok(MemoryStatus::Expired),
        _ => Err(AppError::database(format!("unknown memory status: {value}"))),
    }
}

fn index_policy_to_str(policy: &IndexPolicy) -> &'static str {
    match policy {
        IndexPolicy::None => "none",
        IndexPolicy::Vector => "vector",
    }
}

fn index_policy_from_str(value: &str) -> AppResult<IndexPolicy> {
    match value {
        "none" => Ok(IndexPolicy::None),
        "vector" => Ok(IndexPolicy::Vector),
        _ => Err(AppError::database(format!(
            "unknown memory index policy: {value}"
        ))),
    }
}
