use std::sync::Arc;

use application::{
    app_error::AppResult,
    common::agent::KnowledgeBasePort,
    meal::MealRecordRepositoryPort,
    user::UserProfileRepositoryPort,
};
use tokio::sync::Mutex;

use crate::persistence::{DbRepos, PersistenceBackend};

use super::{
    connect_sqlite, db_err, SqliteAdviceRepo, SqliteMealRepo, SqliteNutritionRepo, SqliteUserRepo,
};

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
        let meal_repo_impl = Arc::new(SqliteMealRepo::new(db.clone()));
        let advice_repo_impl = Arc::new(SqliteAdviceRepo::new(db.clone()));
        let nutrition_repo = Arc::new(SqliteNutritionRepo::new(db));

        let user_repo: Arc<dyn UserProfileRepositoryPort> = user_repo_impl;
        let meal_repo: Arc<dyn MealRecordRepositoryPort> = meal_repo_impl;
        let advice_repo: Arc<dyn KnowledgeBasePort> = advice_repo_impl;

        Ok(DbRepos {
            user_repo,
            meal_repo,
            advice_repo,
            nutrition_repo,
        })
    }
}
