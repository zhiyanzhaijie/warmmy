use async_trait::async_trait;
use application::common::agent::CollaborationPort;

#[derive(Debug, Clone)]
pub struct HitlCollaboration {
    risky_action_keywords: Vec<&'static str>,
}

impl Default for HitlCollaboration {
    fn default() -> Self {
        Self {
            risky_action_keywords: vec![
                "delete",
                "payment",
                "transfer",
                "deploy",
                "overwrite",
                "execute",
            ],
        }
    }
}

#[async_trait]
impl CollaborationPort for HitlCollaboration {
    async fn require_human_confirmation(
        &self,
        action: &str,
        _reason: &str,
    ) -> Result<bool, String> {
        let lower = action.to_lowercase();
        let requires_hitl = self
            .risky_action_keywords
            .iter()
            .any(|keyword| lower.contains(keyword));
        if requires_hitl {
            return Ok(false);
        }
        Ok(true)
    }
}
