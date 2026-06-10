use async_trait::async_trait;
use domain::UserId;

use crate::app_error::AppResult;

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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug)]
pub struct MemoryObservation {
    pub user_id: UserId,
    pub session_id: String,
    pub reference_date: String,
    pub user_input: String,
    pub assistant_output: String,
}

#[async_trait]
pub trait MemoryExtractor: Send + Sync {
    async fn extract(&self, observation: MemoryObservation) -> AppResult<Vec<MemoryCandidate>>;
}

#[derive(Clone, Default)]
pub struct NoopMemoryExtractor;

#[async_trait]
impl MemoryExtractor for NoopMemoryExtractor {
    async fn extract(&self, _observation: MemoryObservation) -> AppResult<Vec<MemoryCandidate>> {
        Ok(Vec::new())
    }
}

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

#[async_trait]
pub trait MemoryIndex: Send + Sync {
    async fn put(&self, record: &MemoryRecord) -> AppResult<()>;
}

#[derive(Clone, Default)]
pub struct MemoryPolicy;

impl MemoryPolicy {
    pub fn new() -> Self {
        Self
    }

    pub fn accepts(&self, candidate: &MemoryCandidate) -> bool {
        !candidate.content.trim().is_empty()
            && matches!(
                candidate.status,
                MemoryStatus::Candidate | MemoryStatus::Confirmed
            )
            && matches!(
                candidate.index_policy,
                IndexPolicy::None | IndexPolicy::Vector
            )
    }

    pub fn indexes(&self, record: &MemoryRecord) -> bool {
        record.status == MemoryStatus::Confirmed && record.index_policy == IndexPolicy::Vector
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromotionDecision {
    Promote,
    Dismiss,
    Keep,
}

#[derive(Clone, Default)]
pub struct PromotionPolicy;

impl PromotionPolicy {
    pub fn new() -> Self {
        Self
    }

    pub fn decide(&self, record: &MemoryRecord) -> PromotionDecision {
        if record.status != MemoryStatus::Candidate {
            return PromotionDecision::Keep;
        }

        let content = record.content.trim();
        if content.is_empty() || contains_secret_like_content(content) {
            return PromotionDecision::Dismiss;
        }

        match record.form {
            LongTermMemoryForm::Procedural => PromotionDecision::Promote,
            LongTermMemoryForm::Semantic => PromotionDecision::Promote,
            LongTermMemoryForm::Episodic => {
                if looks_like_meal_only(content) {
                    PromotionDecision::Keep
                } else {
                    PromotionDecision::Promote
                }
            }
        }
    }
}

pub fn is_conflict(existing: &MemoryRecord, candidate: &MemoryRecord) -> bool {
    if existing.user_id != candidate.user_id
        || existing.status != MemoryStatus::Confirmed
        || !matches!(
            candidate.status,
            MemoryStatus::Candidate | MemoryStatus::Confirmed
        )
    {
        return false;
    }

    let (Some(existing_claim), Some(candidate_claim)) =
        (existing.claim.as_ref(), candidate.claim.as_ref())
    else {
        return false;
    };

    existing_claim.subject == candidate_claim.subject
        && existing_claim.predicate == candidate_claim.predicate
        && is_single_valued_predicate(&candidate_claim.predicate)
}

pub fn is_duplicate(existing: &MemoryRecord, candidate: &MemoryRecord) -> bool {
    if existing.user_id != candidate.user_id
        || existing.status != MemoryStatus::Confirmed
        || !matches!(
            candidate.status,
            MemoryStatus::Candidate | MemoryStatus::Confirmed
        )
        || existing.form != candidate.form
        || existing.scope != candidate.scope
    {
        return false;
    }

    if existing.claim.is_some() || candidate.claim.is_some() {
        return existing.claim == candidate.claim
            && normalize_memory_content(&existing.content)
                == normalize_memory_content(&candidate.content);
    }

    normalize_memory_content(&existing.content) == normalize_memory_content(&candidate.content)
}

fn contains_secret_like_content(content: &str) -> bool {
    let content = content.to_ascii_lowercase();
    [
        "api key",
        "apikey",
        "password",
        "token",
        "验证码",
        "密码",
        "支付密码",
        "身份证",
    ]
    .iter()
    .any(|needle| content.contains(needle))
}

fn looks_like_meal_only(content: &str) -> bool {
    let has_meal_word = [
        "早餐", "午餐", "晚餐", "早饭", "午饭", "晚饭", "吃了", "喝了",
    ]
    .iter()
    .any(|word| content.contains(word));
    let has_experience_word = [
        "去了", "见了", "一起", "感觉", "舒服", "难受", "受伤", "咬到", "压力", "矛盾", "姨妈",
        "朋友", "家人", "同事",
    ]
    .iter()
    .any(|word| content.contains(word));

    has_meal_word && !has_experience_word
}

fn is_single_valued_predicate(predicate: &str) -> bool {
    matches!(
        predicate,
        "home_area"
            | "preferred_name"
            | "assistant_name"
            | "health_goal"
            | "diet_style"
            | "current_location"
    )
}

fn normalize_memory_content(content: &str) -> String {
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
        .to_ascii_lowercase()
}
