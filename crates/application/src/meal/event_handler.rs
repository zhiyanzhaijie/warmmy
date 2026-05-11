use domain::meal::event::MealRecorded;

use crate::app_error::AppResult;

#[derive(Clone, Default)]
pub struct MealEventHandler;

impl MealEventHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_meal_recorded(&self, _event: &MealRecorded) -> AppResult<()> {
        Ok(())
    }
}
