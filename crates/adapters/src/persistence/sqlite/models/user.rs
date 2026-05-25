#[derive(Debug, Clone, toasty::Model)]
pub struct UserProfileRow {
    #[key]
    pub id: String,
    pub display_name: String,
    pub introduction: String,
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
