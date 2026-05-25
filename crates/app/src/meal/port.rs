use async_trait::async_trait;
use domain::{FoodNutritionReference, MealRecord};

#[async_trait]
pub trait MealRecordRepositoryPort: Send + Sync {
    async fn save_meal(&self, meal: &MealRecord) -> Result<(), String>;
}

#[async_trait]
pub trait FoodNutritionReferenceRepositoryPort: Send + Sync {
    async fn upsert_reference(&self, reference: &FoodNutritionReference) -> Result<(), String>;

    async fn find_reference_by_name(
        &self,
        name: &str,
    ) -> Result<Option<FoodNutritionReference>, String>;
}
