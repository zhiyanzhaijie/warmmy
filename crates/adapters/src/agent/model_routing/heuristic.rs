use async_trait::async_trait;
use application::common::agent::ModelRoutingPort;

#[derive(Debug, Clone)]
pub struct HeuristicModelRouter {
    fast_model: String,
    strong_model: String,
}

impl Default for HeuristicModelRouter {
    fn default() -> Self {
        Self {
            fast_model: "gpt-4o-mini".to_string(),
            strong_model: "gpt-4.1".to_string(),
        }
    }
}

#[async_trait]
impl ModelRoutingPort for HeuristicModelRouter {
    async fn select_model(&self, task: &str) -> Result<String, String> {
        let lower = task.to_lowercase();
        let complex = task.chars().count() > 200
            || ["plan", "analyze", "reason", "multi-step", "复杂", "规划"]
                .iter()
                .any(|k| lower.contains(k));
        if complex {
            return Ok(self.strong_model.clone());
        }
        Ok(self.fast_model.clone())
    }
}
