use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use application::meal::MealRecordRepositoryPort;
use domain::MealRecord;

use super::models::MealRecordRow;

#[derive(Clone)]
pub struct PsqlMealRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl PsqlMealRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MealRecordRepositoryPort for PsqlMealRepo {
    async fn save_meal(&self, meal: &MealRecord) -> Result<(), String> {
        let foods_json = serde_json::to_string(&meal.foods).map_err(|err| err.to_string())?;
        let nutrition_json =
            serde_json::to_string(&meal.nutrition).map_err(|err| err.to_string())?;
        let mut db = self.db.lock().await;

        toasty::create!(MealRecordRow {
            user_id: meal.user_id.as_str().to_string(),
            day_cycle: meal.day_cycle.as_str().to_string(),
            foods_json: foods_json,
            nutrition_json: nutrition_json,
        })
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}
