use crate::meal::ProposeMealLogResult;
use domain::UserId;

#[derive(Debug, Clone)]
pub struct MealIntakeInput {
    pub user_id: UserId,
    pub session_id: String,
    pub day_cycle: String,
    pub foods: Vec<MealIntakeFood>,
}

#[derive(Debug, Clone)]
pub struct MealIntakeFood {
    pub name: String,
    pub grams: f32,
    pub amount_confidence: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct MealIntakeOutput {
    pub result: ProposeMealLogResult,
    pub critiques: Vec<MealIntakeCritique>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealIntakeStage {
    PortionNormalized,
    PendingProposed,
}

#[derive(Debug, Clone)]
pub struct MealIntakeCritique {
    pub food_name: String,
    pub kind: MealIntakeCritiqueKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealIntakeCritiqueKind {
    LowAmountConfidence,
    UnusualAmount,
}
