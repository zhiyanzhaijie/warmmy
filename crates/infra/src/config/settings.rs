use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub providers: HashMap<String, LlmProviderConfig>,
    pub models: HashMap<String, LlmModelConfig>,
    pub routing: LlmRoutingConfig,
    pub enable_image_parsing: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmProviderConfig {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmModelConfig {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmRoutingConfig {
    pub reasoning: String,
    pub perception: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedLlmRoute {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let path = format!("{}/src/config/toml/{env}.toml", env!("CARGO_MANIFEST_DIR"));

        ConfigBuilder::builder()
            .add_source(File::new(&path, config::FileFormat::Toml))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }
}
impl LlmConfig {
    pub fn resolve_route(&self, alias: &str) -> Result<ResolvedLlmRoute, String> {
        let model_cfg = self
            .models
            .get(alias)
            .ok_or_else(|| format!("llm model alias not found: {}", alias))?;
        let provider_cfg = self
            .providers
            .get(&model_cfg.provider)
            .ok_or_else(|| format!("llm provider not found: {}", model_cfg.provider))?;

        Ok(ResolvedLlmRoute {
            provider: model_cfg.provider.clone(),
            base_url: provider_cfg.base_url.clone(),
            api_key: provider_cfg.api_key.clone(),
            model: model_cfg.model.clone(),
        })
    }
}
