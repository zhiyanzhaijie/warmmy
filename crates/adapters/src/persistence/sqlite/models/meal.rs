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
