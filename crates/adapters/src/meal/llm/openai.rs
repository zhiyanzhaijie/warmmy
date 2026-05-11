use async_trait::async_trait;
use domain::FoodItem;

use application::meal::LlmPort;

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
impl LlmPort for OpenAiCompatibleLlm {
    async fn parse_meal_from_text(&self, content: &str) -> Result<Vec<FoodItem>, String> {
        let foods = content
            .split([',', '，'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|name| FoodItem::new(name, 1.0, "份"))
            .collect::<Vec<_>>();
        Ok(foods)
    }

    async fn parse_meal_from_image(&self, _image_url: &str) -> Result<Vec<FoodItem>, String> {
        Err("image parsing is not configured yet".to_string())
    }

    async fn generate_advice(&self, prompt: &str) -> Result<String, String> {
        Ok(format!("模型({})建议：{}", self.model, prompt))
    }
}
