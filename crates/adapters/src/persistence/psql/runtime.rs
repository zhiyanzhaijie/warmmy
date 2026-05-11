use std::sync::Arc;

use application::{
    app_error::AppResult,
    advice::KnowledgeBasePort,
    meal::MealRecordRepositoryPort,
    user::UserProfileRepositoryPort,
};
use tokio::sync::Mutex;

use crate::persistence::{DbRepos, PersistenceBackend};

use super::{connect_psql, db_err, PsqlAdviceRepo, PsqlMealRepo, PsqlNutritionRepo, PsqlUserRepo};

pub struct PsqlBackend;

#[async_trait::async_trait]
impl PersistenceBackend for PsqlBackend {
    fn name(&self) -> &'static str {
        "postgresql+toasty"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.starts_with("postgres://") || url.starts_with("postgresql://")
    }

    async fn build_repos(&self, database_url: &str) -> AppResult<DbRepos> {
        let db = connect_psql(database_url).await.map_err(db_err)?;
        let db = Arc::new(Mutex::new(db));

        let user_repo_impl = Arc::new(PsqlUserRepo::new(db.clone()));
        let meal_repo_impl = Arc::new(PsqlMealRepo::new(db.clone()));
        let advice_repo_impl = Arc::new(PsqlAdviceRepo::new(db.clone()));
        let nutrition_repo = Arc::new(PsqlNutritionRepo::new(db));

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
