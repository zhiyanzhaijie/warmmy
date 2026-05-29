use async_trait::async_trait;
use domain::{
    FoodNutritionReference, MealDayFinalization, MealDaySummary, MealRecord, PendingMealLog,
    PendingMealLogId, UserId,
};

#[async_trait]
pub trait MealRecordRepositoryPort: Send + Sync {
    async fn save_meal(&self, meal: &MealRecord) -> Result<(), String>;

    async fn list_meals(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<MealRecord>, String>;
}

#[async_trait]
pub trait MealDayFinalizationRepositoryPort: Send + Sync {
    async fn save_finalization(&self, finalization: &MealDayFinalization) -> Result<(), String>;

    async fn find_finalization(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Option<MealDayFinalization>, String>;
}

#[async_trait]
pub trait MealDaySummaryRepositoryPort: Send + Sync {
    async fn save_summary(&self, summary: &MealDaySummary) -> Result<(), String>;

    async fn find_summary(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Option<MealDaySummary>, String>;

    async fn list_summaries(&self, user_id: &UserId) -> Result<Vec<MealDaySummary>, String>;
}

#[async_trait]
pub trait PendingMealLogRepositoryPort: Send + Sync {
    async fn save_pending_meal(&self, pending: &PendingMealLog) -> Result<(), String>;

    async fn find_pending_meal(
        &self,
        user_id: &UserId,
        id: &PendingMealLogId,
    ) -> Result<Option<PendingMealLog>, String>;

    async fn list_pending_meals(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<PendingMealLog>, String>;

    async fn delete_pending_meal(
        &self,
        user_id: &UserId,
        id: &PendingMealLogId,
    ) -> Result<(), String>;
}

#[async_trait]
pub trait FoodNutritionReferenceRepositoryPort: Send + Sync {
    async fn upsert_reference(&self, reference: &FoodNutritionReference) -> Result<(), String>;

    async fn find_reference_by_name(
        &self,
        name: &str,
    ) -> Result<Option<FoodNutritionReference>, String>;
}
