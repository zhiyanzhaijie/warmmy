use domain::UserId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LongTermMemoryForm {
    Semantic,
    Episodic,
    Procedural,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryScope {
    Day { date: String },
    UserGlobal,
    Temporary { expires_at: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemorySource {
    UserUtterance { session_id: String },
    AgentReflection { run_id: String },
    DomainEvent { entity: String, id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryStatus {
    Candidate,
    Confirmed,
    Superseded,
    Rejected,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexPolicy {
    None,
    Vector,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryClaim {
    pub subject: String,
    pub predicate: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryRecord {
    pub id: String,
    pub user_id: UserId,
    pub form: LongTermMemoryForm,
    pub scope: MemoryScope,
    pub source: MemorySource,
    pub status: MemoryStatus,
    pub index_policy: IndexPolicy,
    pub content: String,
    pub search_text: String,
    pub confidence: f32,
    pub claim: Option<MemoryClaim>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCandidate {
    pub user_id: UserId,
    pub form: LongTermMemoryForm,
    pub scope: MemoryScope,
    pub source: MemorySource,
    pub status: MemoryStatus,
    pub index_policy: IndexPolicy,
    pub content: String,
    pub search_text: String,
    pub confidence: f32,
    pub claim: Option<MemoryClaim>,
}
