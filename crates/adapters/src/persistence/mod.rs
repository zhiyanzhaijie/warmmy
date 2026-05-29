mod app_error_impl;
pub mod ephemeral_image_store;
pub mod sqlite;

use std::sync::Arc;

use app::{
    app_error::{AppError, AppResult},
    meal::{
        FoodNutritionReferenceRepositoryPort, MealDayFinalizationRepositoryPort,
        MealDaySummaryRepositoryPort, MealRecordRepositoryPort, PendingMealLogRepositoryPort,
    },
    user::{
        DiningCompanionRepositoryPort, SecretStorePort, UserAIConfigRepositoryPort,
        UserHealthExpectationRepositoryPort, UserPreferencesRepositoryPort,
        UserProfileRepositoryPort,
    },
};

use tokio::sync::Mutex;

use app::conversation::{ChatMessageRepositoryPort, EphemeralImageStorePort};

pub struct DbRepos {
    pub db: Arc<Mutex<toasty::Db>>,
    pub user_repo: Arc<dyn UserProfileRepositoryPort>,
    pub user_expectation_repo: Arc<dyn UserHealthExpectationRepositoryPort>,
    pub user_preferences_repo: Arc<dyn UserPreferencesRepositoryPort>,
    pub dining_companion_repo: Arc<dyn DiningCompanionRepositoryPort>,
    pub user_ai_config_repo: Arc<dyn UserAIConfigRepositoryPort>,
    pub secret_store: Arc<dyn SecretStorePort>,
    pub chat_repo: Arc<dyn ChatMessageRepositoryPort>,
    pub ephemeral_image_store: Arc<dyn EphemeralImageStorePort>,
    pub meal_repo: Arc<dyn MealRecordRepositoryPort>,
    pub pending_meal_repo: Arc<dyn PendingMealLogRepositoryPort>,
    pub meal_day_finalization_repo: Arc<dyn MealDayFinalizationRepositoryPort>,
    pub meal_day_summary_repo: Arc<dyn MealDaySummaryRepositoryPort>,
    pub food_nutrition_reference_repo: Arc<dyn FoodNutritionReferenceRepositoryPort>,
}

#[async_trait::async_trait]
pub trait PersistenceBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, url: &str) -> bool;
    async fn build_repos(&self, database_url: &str) -> AppResult<DbRepos>;
}

pub async fn build_repos_by_url(database_url: &str) -> AppResult<DbRepos> {
    let sqlite = sqlite::SqliteBackend;
    let backends: [&dyn PersistenceBackend; 1] = [&sqlite];
    for backend in backends {
        if backend.can_handle(database_url) {
            return backend.build_repos(database_url).await;
        }
    }

    Err(AppError::internal(format!(
        "Unsupported database url for persistence backend: {database_url}"
    )))
}
