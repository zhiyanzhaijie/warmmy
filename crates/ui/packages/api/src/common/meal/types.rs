#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct FoodItemDTO {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
    #[serde(default)]
    pub estimated_grams: Option<f32>,
    #[serde(default)]
    pub amount_confidence: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct NutritionDTO {
    pub calories: f32,
    pub protein_g: f32,
    pub fat_g: f32,
    pub carbs_g: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct FoodNutritionReferenceDTO {
    pub id: String,
    pub name: String,
    pub terms: Vec<String>,
    pub basis_quantity: f32,
    pub basis_unit: String,
    pub nutrition: NutritionDTO,
    pub status: String,
    pub source: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MealRecordDTO {
    pub session_id: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemDTO>,
    pub nutrition: NutritionDTO,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct PendingMealLogDTO {
    pub id: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemDTO>,
    pub status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ConfirmPendingMealInput {
    pub pending_id: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemDTO>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DiscardPendingMealsOutput {
    pub discarded_ids: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MealDayFinalizationDTO {
    pub session_id: String,
    pub finalized_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MealDaySummaryDTO {
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
