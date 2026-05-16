use std::sync::Arc;
use crate::common::agent::SessionMemoryPort;

use domain::UserId;

use crate::app_error::{AppError, AppResult};

#[derive(Clone)]
pub struct MealQueryHandler {
    memory: Arc<dyn SessionMemoryPort>,
}

impl MealQueryHandler {
    pub fn new(memory: Arc<dyn SessionMemoryPort>) -> Self {
        Self { memory }
    }

    pub async fn recent_dialogue(&self, user_id: &UserId) -> AppResult<Vec<String>> {
        self.memory
            .get_recent_dialogue(user_id)
            .await
            .map_err(AppError::upstream)
    }
}
