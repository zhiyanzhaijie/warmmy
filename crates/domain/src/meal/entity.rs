use serde::{Deserialize, Serialize};

use super::{DayCycle, Nutrition, PendingMealLogId, PendingMealLogStatus};
use crate::UserId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FoodItem {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}

impl FoodItem {
    pub fn new(name: impl Into<String>, quantity: f32, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            quantity,
            unit: unit.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MealRecord {
    pub user_id: UserId,
    pub session_id: String,
    pub day_cycle: DayCycle,
    pub foods: Vec<FoodItem>,
    pub nutrition: Nutrition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingMealLog {
    pub id: PendingMealLogId,
    pub user_id: UserId,
    pub session_id: String,
    pub day_cycle: DayCycle,
    pub foods: Vec<FoodItem>,
    pub nutrition: Nutrition,
    pub status: PendingMealLogStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MealDayFinalization {
    pub user_id: UserId,
    pub session_id: String,
    pub finalized_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MealDaySummary {
    pub user_id: UserId,
    pub session_id: String,
    pub content: String,
    pub nutrition_score: f32,
    pub expectation_match_score: f32,
    pub overall_score: f32,
    pub metrics_json: String,
    pub finalized_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl MealRecord {
    pub fn new(
        user_id: UserId,
        session_id: impl Into<String>,
        day_cycle: impl Into<String>,
        foods: Vec<FoodItem>,
        nutrition: Nutrition,
    ) -> Self {
        Self {
            user_id,
            session_id: session_id.into(),
            day_cycle: DayCycle::new_unchecked(day_cycle),
            foods,
            nutrition,
        }
    }
}
