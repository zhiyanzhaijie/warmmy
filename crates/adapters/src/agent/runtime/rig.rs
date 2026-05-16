use rig::client::CompletionClient;
use rig::completion::Prompt;

use crate::agent::provider::openai;

#[derive(Clone)]
pub struct RigProviderConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Clone)]
pub struct RigRuntimeConfig {
    pub model: String,
    pub provider: RigProviderConfig,
}

#[derive(Clone)]
pub struct RigRuntime {
    config: RigRuntimeConfig,
}

impl RigRuntime {
    pub fn new(config: RigRuntimeConfig) -> Self {
        Self { config }
    }

    pub async fn complete(&self, preamble: &str, prompt: &str) -> Result<String, String> {
        match self.config.provider.name.as_str() {
            "openai" | "deepseek" => {
                let client =
                    openai::client(&self.config.provider.api_key, &self.config.provider.base_url)?;
                let agent = client
                    .agent(self.config.model.as_str())
                    .preamble(preamble)
                    .build();
                agent.prompt(prompt).await.map_err(|err| err.to_string())
            }
            provider => Err(format!("unsupported provider: {}", provider)),
        }
    }
}
