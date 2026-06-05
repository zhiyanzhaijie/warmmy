use std::sync::Arc;

use crate::app_error::{AppError, AppResult};
use crate::meal::{FoodNutritionReferenceRepositoryPort, MealRecordRepositoryPort};
use domain::{FoodNutritionReference, MealRecord, UserId};

#[derive(Clone)]
pub struct MealQueryHandler {
    meals: Arc<dyn MealRecordRepositoryPort>,
    food_nutrition_references: Arc<dyn FoodNutritionReferenceRepositoryPort>,
}

impl MealQueryHandler {
    pub fn new(
        meals: Arc<dyn MealRecordRepositoryPort>,
        food_nutrition_references: Arc<dyn FoodNutritionReferenceRepositoryPort>,
    ) -> Self {
        Self {
            meals,
            food_nutrition_references,
        }
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

    pub async fn list_food_nutrition_references(
        &self,
    ) -> AppResult<Vec<FoodNutritionReference>> {
        self.food_nutrition_references
            .list_references()
            .await
            .map_err(AppError::upstream)
    }
}
