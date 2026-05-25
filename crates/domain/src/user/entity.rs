use serde::{Deserialize, Serialize};

use super::{
    AppTheme, ExpectationSource, HealthExpectationId, HealthExpectationKind,
    HealthExpectationStatus, PreferenceConfidence, UserId,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub id: UserId,
    pub display_name: String,
    pub introduction: String,
    pub allergies: Vec<String>,
}

impl UserProfile {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: UserId::new_unchecked(id),
            display_name: display_name.into(),
            introduction: String::new(),
            allergies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserHealthExpectation {
    pub id: HealthExpectationId,
    pub user_id: UserId,
    pub title: String,
    pub summary: String,
    pub kind: HealthExpectationKind,
    pub status: HealthExpectationStatus,
    pub source: ExpectationSource,
    pub priority: u8,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserPreferences {
    pub user_id: UserId,
    pub app: AppPreferences,
    pub diet: DietaryPreferences,
}

impl UserPreferences {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            app: AppPreferences::default(),
            diet: DietaryPreferences::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppPreferences {
    pub theme: Option<AppTheme>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DietaryPreferences {
    pub preferred_cuisines: Vec<CuisinePreference>,
    pub avoided_cuisines: Vec<CuisinePreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuisinePreference {
    pub cuisine: String,
    pub confidence: PreferenceConfidence,
}
