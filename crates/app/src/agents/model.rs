use async_trait::async_trait;

use crate::app_error::AppResult;
use crate::user::ResolvedAIModelConfig;

#[derive(Clone)]
pub struct TextPromptRequest {
    pub model: ResolvedAIModelConfig,
    pub preamble: String,
    pub prompt: String,
}

#[async_trait]
pub trait TextModelGateway: Send + Sync {
    async fn prompt_text(&self, request: TextPromptRequest) -> AppResult<String>;
}

pub trait AgentServiceProgress: Send + Sync {
    fn status(&self, label: &'static str);
}
