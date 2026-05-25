pub use crate::agent::memory::long_term::rag::RagConfig;

#[derive(Clone)]
pub struct AgentConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub rag: RagConfig,
}
