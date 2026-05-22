use async_trait::async_trait;
use domain::MealRecord;

#[async_trait]
pub trait MealRecordRepositoryPort: Send + Sync {
    async fn save_meal(&self, meal: &MealRecord) -> Result<(), String>;
}
