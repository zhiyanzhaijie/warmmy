use async_trait::async_trait;
use application::common::agent::{PerceptionInput, PerceptionPort, ReasoningPort};
use crate::agent::runtime::rig::{RigRuntime, RigRuntimeConfig};

#[derive(Clone)]
pub struct RuntimeLlmAdapter {
    runtime: RigRuntime,
    enable_image_parsing: bool,
}

impl RuntimeLlmAdapter {
    pub fn new(config: RigRuntimeConfig, enable_image_parsing: bool) -> Self {
        Self {
            runtime: RigRuntime::new(config),
            enable_image_parsing,
        }
    }

}

#[async_trait]
impl ReasoningPort for RuntimeLlmAdapter {
    async fn complete_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        self.runtime.complete(system_prompt, user_prompt).await
    }
}

#[async_trait]
impl PerceptionPort for RuntimeLlmAdapter {
    async fn perceive(&self, input: PerceptionInput, instruction: &str) -> Result<String, String> {
        let preamble = "你是结构化感知助手。你必须输出符合要求的结构化内容，不要输出其它解释。";
        let prompt = match input {
            PerceptionInput::Text(content) => format!("{instruction}\n输入文本：{content}"),
            PerceptionInput::ImageUrl(image_url) => {
                if !self.enable_image_parsing {
                    return Err("image parsing is disabled".to_string());
                }
                format!("{instruction}\n输入图片链接：{image_url}")
            }
        };
        self.runtime.complete(preamble, &prompt).await
    }
}
