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
pub struct MealRecordRow {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: String,
    pub day_cycle: String,
    pub foods_json: String,
    pub nutrition_json: String,
}
