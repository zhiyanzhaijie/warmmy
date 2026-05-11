use serde::{Deserialize, Serialize};

use super::{HealthGoal, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub id: UserId,
    pub display_name: String,
    pub introduction: String,
    pub health_goal: HealthGoal,
    pub allergies: Vec<String>,
}

impl UserProfile {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        health_goal: impl Into<String>,
    ) -> Self {
        Self {
            id: UserId::new_unchecked(id),
            display_name: display_name.into(),
            introduction: String::new(),
            health_goal: HealthGoal::new(health_goal),
            allergies: Vec::new(),
        }
    }
}
