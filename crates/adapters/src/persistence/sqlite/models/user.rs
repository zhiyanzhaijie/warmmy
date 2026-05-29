#[derive(Debug, Clone, toasty::Model)]
pub struct UserProfileRow {
    #[key]
    pub id: String,
    pub display_name: String,
    pub introduction: String,
    pub allergies_json: String,
    pub gender: Option<String>,
    pub age: Option<i32>,
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
pub struct DiningCompanionRow {
    #[key]
    pub id: String,
    #[index]
    pub owner_user_id: String,
    pub display_name: String,
    pub relationship: Option<String>,
    pub introduction: String,
    pub dietary_preferences_json: String,
    pub health_notes_json: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserAIProviderRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    pub kind: String,
    pub name: String,
    pub base_url: String,
    pub secret_ref: Option<String>,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserAIRouteRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    pub capability: String,
    pub provider_id: String,
    pub model: String,
    pub embedding_ndims: Option<i32>,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserSecretRow {
    #[key]
    pub id: String,
    #[index]
    pub scope: String,
    pub secret_value: String,
    pub updated_at: String,
}
