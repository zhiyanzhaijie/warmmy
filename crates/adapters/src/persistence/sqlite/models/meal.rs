#[derive(Debug, Clone, toasty::Model)]
pub struct MealRecordRow {
    #[key]
    #[auto]
    pub id: i32,
    #[index]
    pub user_id: String,
    #[index]
    pub session_id: String,
    pub day_cycle: String,
    pub foods_json: String,
    pub nutrition_json: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct PendingMealLogRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    #[index]
    pub session_id: String,
    pub day_cycle: String,
    pub foods_json: String,
    pub nutrition_json: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct FoodNutritionReferenceRow {
    #[key]
    pub id: String,
    pub reference_id: String,
    pub labels_json: String,
    pub aliases_json: String,
    pub basis_quantity: f32,
    pub basis_unit: String,
    pub nutrition_json: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct MealDayFinalizationRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    #[index]
    pub session_id: String,
    pub finalized_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct MealDaySummaryRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    #[index]
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
