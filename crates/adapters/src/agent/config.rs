use app::user::ResolvedAIModelConfig;

pub use crate::agent::memory::long_term::rag::RagConfig;

#[derive(Clone)]
pub struct AgentModelConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl From<ResolvedAIModelConfig> for AgentModelConfig {
    fn from(value: ResolvedAIModelConfig) -> Self {
        Self {
            provider: value.provider.as_str().to_string(),
            base_url: value.base_url,
            api_key: value.api_key,
            model: value.model,
        }
    }
}
