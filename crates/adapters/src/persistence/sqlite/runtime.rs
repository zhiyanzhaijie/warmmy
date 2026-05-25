use std::sync::Arc;

use app::{
    app_error::AppResult,
    conversation::ChatMessageRepositoryPort,
    meal::{FoodNutritionReferenceRepositoryPort, MealRecordRepositoryPort},
    user::{
        UserHealthExpectationRepositoryPort, UserPreferencesRepositoryPort,
        UserProfileRepositoryPort,
    },
};
use tokio::sync::Mutex;

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

        let user_repo_impl = Arc::new(SqliteUserRepo::new(db.clone()));
        let chat_repo_impl = Arc::new(SqliteChatMessageRepo::new(db.clone()));
        let meal_repo_impl = Arc::new(SqliteMealRepo::new(db.clone()));

        let user_repo: Arc<dyn UserProfileRepositoryPort> = user_repo_impl.clone();
        let user_expectation_repo: Arc<dyn UserHealthExpectationRepositoryPort> =
            user_repo_impl.clone();
        let user_preferences_repo: Arc<dyn UserPreferencesRepositoryPort> = user_repo_impl;
        let chat_repo: Arc<dyn ChatMessageRepositoryPort> = chat_repo_impl;
        let meal_repo: Arc<dyn MealRecordRepositoryPort> = meal_repo_impl.clone();
        let food_nutrition_reference_repo: Arc<dyn FoodNutritionReferenceRepositoryPort> =
            meal_repo_impl;

        Ok(DbRepos {
            db,
            user_repo,
            user_expectation_repo,
            user_preferences_repo,
            chat_repo,
            meal_repo,
            food_nutrition_reference_repo,
        })
    }
}
