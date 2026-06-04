use crate::agent::memory::types::{IndexPolicy, MemoryCandidate, MemoryStatus};

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
            && matches!(candidate.index_policy, IndexPolicy::None | IndexPolicy::Vector)
    }

    pub fn indexes(&self, record: &crate::agent::memory::types::MemoryRecord) -> bool {
        record.status == MemoryStatus::Confirmed && record.index_policy == IndexPolicy::Vector
    }
}
