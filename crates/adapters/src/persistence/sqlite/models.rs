#[derive(Debug, Clone, toasty::Model)]
pub struct UserProfileRow {
    #[key]
    pub id: String,
    pub display_name: String,
    pub introduction: String,
    pub health_goal: String,
    pub allergies_json: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserHealthExpectationRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    pub title: String,
    pub summary: String,
    pub kind: String,
    pub status: String,
    pub source_json: String,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserPreferencesRow {
    #[key]
    pub user_id: String,
    pub app_preferences_json: String,
    pub dietary_preferences_json: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct MealRecordRow {
    #[key]
    #[auto]
    pub id: i32,
    #[index]
    pub user_id: String,
    pub day_cycle: String,
    pub foods_json: String,
    pub nutrition_json: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ChatMessageRow {
    #[key]
    #[auto]
    pub id: i32,
    #[index]
    pub user_id: String,
    #[index]
    pub session_id: String,
    pub role: String, // "user" | "assistant"
    pub content: String,
    pub created_at: String, // RFC3339
}
