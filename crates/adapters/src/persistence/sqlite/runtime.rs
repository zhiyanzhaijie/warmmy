use std::sync::Arc;

use app::{
    app_error::{AppError, AppResult},
    conversation::ChatMessageRepositoryPort,
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

use crate::crypto::argon2::Argon2SecretCipher;
use crate::persistence::ephemeral_image_store::InMemoryEphemeralImageStore;
use crate::persistence::{DbRepos, PersistenceBackend};

use super::{connect_sqlite, db_err, SqliteChatMessageRepo, SqliteMealRepo, SqliteUserRepo};

pub struct SqliteBackend;

#[async_trait::async_trait]
impl PersistenceBackend for SqliteBackend {
    fn name(&self) -> &'static str {
        "sqlite+toasty"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.starts_with("sqlite://")
            || url.starts_with("sqlite:")
            || url.ends_with(".db")
            || url.ends_with(".sqlite")
    }

    async fn build_repos(&self, database_url: &str) -> AppResult<DbRepos> {
        let db = connect_sqlite(database_url).await.map_err(db_err)?;
        let db = Arc::new(Mutex::new(db));
        let secret_cipher = Arc::new(
            Argon2SecretCipher::from_database_url(database_url).map_err(AppError::internal)?,
        );

        let user_repo_impl = Arc::new(SqliteUserRepo::new(db.clone(), secret_cipher));
        let chat_repo_impl = Arc::new(SqliteChatMessageRepo::new(db.clone()));
        let ephemeral_image_store_impl = Arc::new(InMemoryEphemeralImageStore::new());
        let meal_repo_impl = Arc::new(SqliteMealRepo::new(db.clone()));

        let user_repo: Arc<dyn UserProfileRepositoryPort> = user_repo_impl.clone();
        let user_expectation_repo: Arc<dyn UserHealthExpectationRepositoryPort> =
            user_repo_impl.clone();
        let user_preferences_repo: Arc<dyn UserPreferencesRepositoryPort> = user_repo_impl.clone();
        let dining_companion_repo: Arc<dyn DiningCompanionRepositoryPort> = user_repo_impl.clone();
        let user_ai_config_repo: Arc<dyn UserAIConfigRepositoryPort> = user_repo_impl.clone();
        let secret_store: Arc<dyn SecretStorePort> = user_repo_impl.clone();
        let chat_repo: Arc<dyn ChatMessageRepositoryPort> = chat_repo_impl;
        let ephemeral_image_store = ephemeral_image_store_impl;
        let meal_repo: Arc<dyn MealRecordRepositoryPort> = meal_repo_impl.clone();
        let pending_meal_repo: Arc<dyn PendingMealLogRepositoryPort> = meal_repo_impl.clone();
        let meal_day_finalization_repo: Arc<dyn MealDayFinalizationRepositoryPort> =
            meal_repo_impl.clone();
        let meal_day_summary_repo: Arc<dyn MealDaySummaryRepositoryPort> = meal_repo_impl.clone();
        let food_nutrition_reference_repo: Arc<dyn FoodNutritionReferenceRepositoryPort> =
            meal_repo_impl;

        Ok(DbRepos {
            db,
            user_repo,
            user_expectation_repo,
            user_preferences_repo,
            dining_companion_repo,
            user_ai_config_repo,
            secret_store,
            chat_repo,
            ephemeral_image_store,
            meal_repo,
            pending_meal_repo,
            meal_day_finalization_repo,
            meal_day_summary_repo,
            food_nutrition_reference_repo,
        })
    }
}
