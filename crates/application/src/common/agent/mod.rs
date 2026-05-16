use async_trait::async_trait;
use domain::UserId;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerceptionInput {
    Text(String),
    ImageUrl(String),
}

#[async_trait]
pub trait ReasoningPort: Send + Sync {
    async fn complete_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String>;
}
#[async_trait]
pub trait PerceptionPort: Send + Sync {
    async fn perceive(&self, input: PerceptionInput, instruction: &str) -> Result<String, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardrailDecision {
    Allow,
    Reject(String),
}

#[async_trait]
pub trait PlanningPort: Send + Sync {
    async fn plan(&self, goal: &str, context: &[String]) -> Result<Vec<String>, String>;
}

#[async_trait]
pub trait ToolExecutionPort: Send + Sync {
    async fn execute_tool(&self, tool_name: &str, payload: &str) -> Result<String, String>;
}

#[async_trait]
pub trait GuardrailsPort: Send + Sync {
    async fn check_input(&self, input: &str) -> Result<GuardrailDecision, String>;
    async fn check_effect(&self, effect: &str) -> Result<GuardrailDecision, String>;
    async fn check_output(&self, output: &str) -> Result<GuardrailDecision, String>;
}

#[async_trait]
pub trait CollaborationPort: Send + Sync {
    async fn require_human_confirmation(
        &self,
        action: &str,
        reason: &str,
    ) -> Result<bool, String>;
}

#[async_trait]
pub trait ModelRoutingPort: Send + Sync {
    async fn select_model(&self, task: &str) -> Result<String, String>;
}

#[async_trait]
pub trait SessionMemoryPort: Send + Sync {
    async fn get_recent_dialogue(&self, user_id: &UserId) -> Result<Vec<String>, String>;
    async fn append_dialogue(&self, user_id: &UserId, message: String) -> Result<(), String>;
}

#[async_trait]
pub trait KnowledgeBasePort: Send + Sync {
    async fn search_user_knowledge(
        &self,
        user_id: &UserId,
        query: &str,
    ) -> Result<Vec<String>, String>;
}
