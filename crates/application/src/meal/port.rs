use async_trait::async_trait;
use domain::{FoodItem, MealRecord, UserId};

#[async_trait]
pub trait LlmPort: Send + Sync {
    async fn parse_meal_from_text(&self, content: &str) -> Result<Vec<FoodItem>, String>;
    async fn parse_meal_from_image(&self, image_url: &str) -> Result<Vec<FoodItem>, String>;
    async fn generate_advice(&self, prompt: &str) -> Result<String, String>;
}

#[async_trait]
pub trait SessionMemoryPort: Send + Sync {
    async fn get_recent_dialogue(&self, user_id: &UserId) -> Result<Vec<String>, String>;
    async fn append_dialogue(&self, user_id: &UserId, message: String) -> Result<(), String>;
}

#[async_trait]
pub trait MealRecordRepositoryPort: Send + Sync {
    async fn save_meal(&self, meal: &MealRecord) -> Result<(), String>;
}
