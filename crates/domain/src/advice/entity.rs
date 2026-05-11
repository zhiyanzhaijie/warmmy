use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MealAdvice {
    pub summary: String,
    pub next_meal_suggestion: String,
    pub warnings: Vec<String>,
}
