use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::meal::{MealCommandHandler, ProposeMealLogCommand, ProposeMealLogResult};
use domain::{FoodItem, PendingMealLogId, UserId};

use crate::agent::services::AgentServiceProgress;

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

#[derive(Clone)]
pub struct MealIntakeChain {
    meal_command: Arc<MealCommandHandler>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
}

impl MealIntakeChain {
    pub fn new(meal_command: Arc<MealCommandHandler>) -> Self {
        Self {
            meal_command,
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: Arc<dyn AgentServiceProgress>) -> Self {
        self.progress = Some(progress);
        self
    }

    pub async fn propose(&self, input: MealIntakeInput) -> AppResult<MealIntakeOutput> {
        let foods = self.normalize_foods(input.foods)?;
        tracing::info!(
            stage = ?MealIntakeStage::PortionNormalized,
            food.count = foods.len(),
            "meal intake chain stage completed"
        );

        let critiques = self.critique(&foods);

        let result = self
            .meal_command
            .propose_meal(ProposeMealLogCommand {
                id: PendingMealLogId::new_unchecked(new_pending_meal_id()),
                user_id: input.user_id,
                session_id: input.session_id,
                day_cycle: input.day_cycle,
                foods,
                nutrition: None,
            })
            .await?;

        tracing::info!(
            stage = ?MealIntakeStage::PendingProposed,
            pending.id = %result.pending.id,
            "meal intake chain stage completed"
        );

        Ok(MealIntakeOutput {
            result,
            critiques,
        })
    }

    fn normalize_foods(&self, foods: Vec<MealIntakeFood>) -> AppResult<Vec<FoodItem>> {
        let foods: Vec<FoodItem> = foods
            .into_iter()
            .filter(|food| !food.name.trim().is_empty())
            .map(|food| {
                let grams = food.grams.max(0.0);
                FoodItem::new(food.name, grams, "g")
                    .with_estimated_amount(Some(grams), food.amount_confidence)
            })
            .collect();

        if foods.is_empty() {
            return Err(AppError::validation("no food items"));
        }

        Ok(foods)
    }

    fn critique(&self, foods: &[FoodItem]) -> Vec<MealIntakeCritique> {
        critique_foods(foods)
    }
}

pub fn critique_foods(foods: &[FoodItem]) -> Vec<MealIntakeCritique> {
    let mut critiques = Vec::new();
    for food in foods {
        if food
            .amount_confidence
            .is_some_and(|confidence| confidence < 0.35)
        {
            critiques.push(MealIntakeCritique {
                food_name: food.name.clone(),
                kind: MealIntakeCritiqueKind::LowAmountConfidence,
                message: "portion estimate confidence is low".to_string(),
            });
        }

        if food.estimated_grams.is_some_and(|grams| grams > 2000.0) {
            critiques.push(MealIntakeCritique {
                food_name: food.name.clone(),
                kind: MealIntakeCritiqueKind::UnusualAmount,
                message: "portion estimate is unusually large".to_string(),
            });
        }
    }

    critiques
}

fn new_pending_meal_id() -> String {
    format!(
        "pml-{}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| chrono::Utc::now().timestamp_micros())
    )
}
