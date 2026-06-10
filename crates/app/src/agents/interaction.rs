#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AgentInteractionRequest {
    pub id: String,
    pub kind: String,
    pub payload: serde_json::Value,
}
