use serde::{Deserialize, Serialize};

use crate::{Nutrition, UserId};

use super::DayCycle;

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
    pub day_cycle: DayCycle,
    pub foods: Vec<FoodItem>,
    pub nutrition: Nutrition,
}

impl MealRecord {
    pub fn new(
        user_id: UserId,
        day_cycle: impl Into<String>,
        foods: Vec<FoodItem>,
        nutrition: Nutrition,
    ) -> Self {
        Self {
            user_id,
            day_cycle: DayCycle::new_unchecked(day_cycle),
            foods,
            nutrition,
        }
    }
}
