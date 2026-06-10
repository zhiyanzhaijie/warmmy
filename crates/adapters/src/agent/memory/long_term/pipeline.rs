use std::sync::Arc;

use app::agents::{MemoryExtractor, MemoryObservation, PromotionDecision, PromotionPolicy};
use app::app_error::AppResult;

use crate::agent::memory::long_term::service::MemoryService;

#[derive(Clone)]
pub struct MemoryPipeline {
    extractor: Arc<dyn MemoryExtractor>,
    service: MemoryService,
    promotion: PromotionPolicy,
}

impl MemoryPipeline {
    pub fn new(extractor: Arc<dyn MemoryExtractor>, service: MemoryService) -> Self {
        Self {
            extractor,
            service,
            promotion: PromotionPolicy::new(),
        }
    }

    pub async fn observe(&self, observation: MemoryObservation) -> AppResult<()> {
        let session_id = observation.session_id.clone();
        let candidates = self.extractor.extract(observation).await?;
        tracing::info!(
            session.id = session_id.as_str(),
            candidate.count = candidates.len(),
            "memory extraction finished"
        );
        for candidate in candidates {
            let Some(record) = self.service.commit(candidate).await? else {
                continue;
            };

            let decision = self.promotion.decide(&record);
            tracing::info!(
                memory.id = record.id.as_str(),
                memory.form = ?record.form,
                memory.scope = ?record.scope,
                memory.status = ?record.status,
                promotion.decision = ?decision,
                "memory promotion decided"
            );

            match decision {
                PromotionDecision::Promote => {
                    self.service.promote(&record.id).await?;
                }
                PromotionDecision::Dismiss => {
                    self.service.dismiss(&record.id).await?;
                }
                PromotionDecision::Keep => {}
            }
        }
        Ok(())
    }
}
