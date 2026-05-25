mod app_error_impl;
pub mod sqlite;

use std::sync::Arc;

use app::{
    app_error::{AppError, AppResult},
    common::agent::KnowledgeBasePort,
    meal::MealRecordRepositoryPort,
    user::{
        UserHealthExpectationRepositoryPort, UserPreferencesRepositoryPort,
        UserProfileRepositoryPort,
    },
};

use tokio::sync::Mutex;

use app::conversation::ChatMessageRepositoryPort;

pub struct DbRepos {
    pub db: Arc<Mutex<toasty::Db>>,
    pub user_repo: Arc<dyn UserProfileRepositoryPort>,
    pub user_expectation_repo: Arc<dyn UserHealthExpectationRepositoryPort>,
    pub user_preferences_repo: Arc<dyn UserPreferencesRepositoryPort>,
    pub chat_repo: Arc<dyn ChatMessageRepositoryPort>,
    pub meal_repo: Arc<dyn MealRecordRepositoryPort>,
    pub advice_repo: Arc<dyn KnowledgeBasePort>,
    pub nutrition_repo: Arc<sqlite::SqliteNutritionRepo>,
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
