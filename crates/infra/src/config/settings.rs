use serde::Deserialize;

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
    pub openai_base_url: String,
    pub model: String,
    pub enable_image_parsing: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://localhost/warmmy".to_string(),
            },
            llm: LlmConfig {
                openai_base_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4o-mini".to_string(),
                enable_image_parsing: false,
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let defaults = Self::default();
        Self {
            database: DatabaseConfig {
                url: std::env::var("APP_DATABASE_URL").unwrap_or(defaults.database.url),
            },
            llm: LlmConfig {
                openai_base_url: std::env::var("APP_OPENAI_BASE_URL")
                    .unwrap_or(defaults.llm.openai_base_url),
                model: std::env::var("APP_OPENAI_MODEL").unwrap_or(defaults.llm.model),
                enable_image_parsing: std::env::var("APP_ENABLE_IMAGE_PARSING")
                    .ok()
                    .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "True"))
                    .unwrap_or(defaults.llm.enable_image_parsing),
            },
        }
    }
}
