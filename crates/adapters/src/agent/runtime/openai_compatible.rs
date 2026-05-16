use async_trait::async_trait;
use application::common::agent::{PerceptionInput, PerceptionPort, ReasoningPort};

#[derive(Clone)]
pub struct OpenAiCompatibleLlm {
    pub model: String,
}

impl OpenAiCompatibleLlm {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait]
impl ReasoningPort for OpenAiCompatibleLlm {
    async fn complete_text(&self, _system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        Ok(format!("模型({})输出：{}", self.model, user_prompt))
    }
}

#[async_trait]
impl PerceptionPort for OpenAiCompatibleLlm {
    async fn perceive(&self, input: PerceptionInput, _instruction: &str) -> Result<String, String> {
        match input {
            PerceptionInput::Text(content) => {
                let items = content
                    .split([',', '，'])
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|name| format!("{{\"name\":\"{}\",\"quantity\":1.0,\"unit\":\"份\"}}", name))
                    .collect::<Vec<_>>()
                    .join(",");
                Ok(format!("[{}]", items))
            }
            PerceptionInput::ImageUrl(_) => Err("image parsing is not configured yet".to_string()),
        }
    }
}
