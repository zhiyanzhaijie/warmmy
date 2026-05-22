use std::sync::Arc;

use crate::meal::MealRecordRepositoryPort;

#[derive(Clone)]
pub struct MealQueryHandler {
    meals: Arc<dyn MealRecordRepositoryPort>,
}

impl MealQueryHandler {
    pub fn new(meals: Arc<dyn MealRecordRepositoryPort>) -> Self {
        Self { meals }
    }
}
