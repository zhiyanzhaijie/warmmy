use crate::agent::memory::types::{LongTermMemoryForm, MemoryRecord, MemoryStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromotionDecision {
    Promote,
    Dismiss,
    Keep,
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
            && normalize_content(&existing.content) == normalize_content(&candidate.content);
    }

    normalize_content(&existing.content) == normalize_content(&candidate.content)
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

fn normalize_content(content: &str) -> String {
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
        .to_ascii_lowercase()
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
    let has_meal_word = ["早餐", "午餐", "晚餐", "早饭", "午饭", "晚饭", "吃了", "喝了"]
        .iter()
        .any(|word| content.contains(word));
    let has_experience_word = [
        "去了", "见了", "一起", "感觉", "舒服", "难受", "受伤", "咬到", "压力", "矛盾",
        "姨妈", "朋友", "家人", "同事",
    ]
    .iter()
    .any(|word| content.contains(word));

    has_meal_word && !has_experience_word
}
