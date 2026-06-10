use async_trait::async_trait;
use domain::{FoodItem, FoodNutritionReference, Nutrition};

use crate::app_error::AppResult;

use super::meal::MealIntakeCritique;

#[derive(Debug, Clone)]
pub struct MealNutritionEstimate {
    pub nutrition: Nutrition,
    pub foods: Vec<ResolvedMealFood>,
    pub gaps: Vec<NutritionKnowledgeGap>,
    pub curated_references: Vec<CuratedNutritionReferenceRecord>,
    pub critiques: Vec<MealIntakeCritique>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMealFood {
    pub item: FoodItem,
    pub reference: Option<FoodNutritionReference>,
}

#[derive(Debug, Clone)]
pub struct NutritionKnowledgeGap {
    pub food_name: String,
    pub reason: NutritionKnowledgeGapReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NutritionKnowledgeGapReason {
    MissingReference,
}

#[derive(Debug, Clone)]
pub struct CuratedNutritionReferenceRecord {
    pub food_name: String,
    pub reference_id: String,
    pub source: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealNutritionStage {
    ReferenceResolved,
    ReferenceMissing,
    ReferenceCurated,
    Estimated,
    Critiqued,
}

#[derive(Debug, Clone)]
pub struct CuratedNutritionReference {
    pub reference: FoodNutritionReference,
    pub confidence: f32,
    pub source: String,
}

#[async_trait]
pub trait NutritionCurator: Send + Sync {
    async fn curate(
        &self,
        food_name: &str,
        estimated_grams: Option<f32>,
    ) -> AppResult<Option<CuratedNutritionReference>>;
}

#[async_trait]
pub trait NutritionReferenceRetriever: Send + Sync {
    async fn retrieve(&self, query: &str) -> AppResult<Option<FoodNutritionReference>>;
    async fn put(&self, reference: &FoodNutritionReference) -> AppResult<()>;
}
