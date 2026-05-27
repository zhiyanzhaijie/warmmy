use std::sync::Arc;

use crate::app_error::{AppError, AppResult};
use crate::meal::MealRecordRepositoryPort;
use domain::{MealRecord, UserId};

#[derive(Clone)]
pub struct MealQueryHandler {
    meals: Arc<dyn MealRecordRepositoryPort>,
}

impl MealQueryHandler {
    pub fn new(meals: Arc<dyn MealRecordRepositoryPort>) -> Self {
        Self { meals }
    }

    pub async fn list_meals(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> AppResult<Vec<MealRecord>> {
        self.meals
            .list_meals(user_id, session_id)
            .await
            .map_err(AppError::upstream)
    }
}
