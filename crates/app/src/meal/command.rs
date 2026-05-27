use std::sync::Arc;

use crate::app_error::{AppError, AppResult};
use crate::meal::{
    estimate_nutrition_from_foods_with_references, FoodNutritionReferenceRepositoryPort,
    MealDayFinalizationRepositoryPort, MealDaySummaryRepositoryPort, MealEventHandler,
    MealRecordRepositoryPort, PendingMealLogRepositoryPort,
};
use crate::user::UserDietaryContextQueryHandler;
use domain::{
    DayCycle, HealthExpectationKind, MealDayFinalization, MealDaySummary, MealRecord, Nutrition,
    PendingMealLog, PendingMealLogId, PendingMealLogStatus, UserId,
};

#[derive(Debug, Clone)]
pub struct LogMealCommand {
    pub user_id: UserId,
    pub session_id: String,
    pub day_cycle: String,
    pub foods: Vec<domain::FoodItem>,
}

#[derive(Debug, Clone)]
pub struct ProposeMealLogCommand {
    pub id: PendingMealLogId,
    pub user_id: UserId,
    pub session_id: String,
    pub day_cycle: String,
    pub foods: Vec<domain::FoodItem>,
}

#[derive(Debug, Clone)]
pub struct UpdatePendingMealLogCommand {
    pub user_id: UserId,
    pub pending_id: PendingMealLogId,
    pub day_cycle: String,
    pub foods: Vec<domain::FoodItem>,
}

#[derive(Debug, Clone)]
pub struct ConfirmMealLogCommand {
    pub user_id: UserId,
    pub pending_id: PendingMealLogId,
}

#[derive(Debug, Clone)]
pub struct RejectMealLogCommand {
    pub user_id: UserId,
    pub pending_id: PendingMealLogId,
}

#[derive(Debug, Clone)]
pub struct FinalizeMealDayCommand {
    pub user_id: UserId,
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct SaveMealDaySummaryCommand {
    pub user_id: UserId,
    pub session_id: String,
    pub content: String,
    pub finalized_at: String,
}

#[derive(Debug, Clone)]
pub struct ProposeMealLogResult {
    pub pending: PendingMealLog,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct LogMealResult {
    pub meal: MealRecord,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct FinalizeMealDayResult {
    pub finalization: MealDayFinalization,
}

#[derive(Debug, Clone)]
pub struct SaveMealDaySummaryResult {
    pub summary: MealDaySummary,
}

#[derive(Clone)]
pub struct MealCommandHandler {
    user_contexts: UserDietaryContextQueryHandler,
    meals: Arc<dyn MealRecordRepositoryPort>,
    pending_meals: Arc<dyn PendingMealLogRepositoryPort>,
    day_finalizations: Arc<dyn MealDayFinalizationRepositoryPort>,
    day_summaries: Arc<dyn MealDaySummaryRepositoryPort>,
    food_nutrition_references: Option<Arc<dyn FoodNutritionReferenceRepositoryPort>>,
    event_handler: Option<MealEventHandler>,
}

impl MealCommandHandler {
    pub fn new(
        user_contexts: UserDietaryContextQueryHandler,
        meals: Arc<dyn MealRecordRepositoryPort>,
        pending_meals: Arc<dyn PendingMealLogRepositoryPort>,
        day_finalizations: Arc<dyn MealDayFinalizationRepositoryPort>,
        day_summaries: Arc<dyn MealDaySummaryRepositoryPort>,
    ) -> Self {
        Self {
            user_contexts,
            meals,
            pending_meals,
            day_finalizations,
            day_summaries,
            food_nutrition_references: None,
            event_handler: None,
        }
    }

    pub fn with_food_nutrition_references(
        mut self,
        food_nutrition_references: Arc<dyn FoodNutritionReferenceRepositoryPort>,
    ) -> Self {
        self.food_nutrition_references = Some(food_nutrition_references);
        self
    }

    pub fn with_event_handler(mut self, event_handler: MealEventHandler) -> Self {
        self.event_handler = Some(event_handler);
        self
    }

    pub async fn handle_meal(&self, input: LogMealCommand) -> AppResult<LogMealResult> {
        let day_cycle = DayCycle::parse(&input.day_cycle)?;
        let context = self
            .user_contexts
            .get_context(&input.user_id)
            .await
            .ok()
            .flatten()
            .ok_or_else(|| AppError::NotFound("user dietary context".to_string()))?;

        self.ensure_day_is_open(&input.user_id, &input.session_id)
            .await?;

        if input.foods.is_empty() {
            return Err(AppError::validation("meal contains no food items"));
        }

        let nutrition = estimate_nutrition_from_foods_with_references(
            &input.foods,
            self.food_nutrition_references.clone(),
        )
        .await;
        let calories = nutrition.calories;
        let meal = MealRecord {
            user_id: input.user_id.clone(),
            session_id: input.session_id.clone(),
            day_cycle: day_cycle.clone(),
            foods: input.foods,
            nutrition,
        };

        self.meals
            .save_meal(&meal)
            .await
            .map_err(AppError::upstream)?;

        if let Some(event_handler) = &self.event_handler {
            let event = domain::meal::event::MealRecorded {
                user_id: meal.user_id.clone(),
                day_cycle: meal.day_cycle.clone(),
            };
            event_handler.handle_meal_recorded(&event).await?;
        }

        let record_summary = format!(
            "已记录{}{}餐：{}，总热量约 {:.0} kcal",
            context.profile.display_name,
            day_cycle.as_str(),
            meal.foods
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>()
                .join("、"),
            calories
        );

        Ok(LogMealResult {
            meal,
            summary: record_summary,
        })
    }

    pub async fn propose_meal(
        &self,
        input: ProposeMealLogCommand,
    ) -> AppResult<ProposeMealLogResult> {
        let day_cycle = DayCycle::parse(&input.day_cycle)?;
        let _context = self
            .user_contexts
            .get_context(&input.user_id)
            .await
            .ok()
            .flatten()
            .ok_or_else(|| AppError::NotFound("user dietary context".to_string()))?;

        self.ensure_day_is_open(&input.user_id, &input.session_id)
            .await?;

        if input.foods.is_empty() {
            return Err(AppError::validation("pending meal contains no food items"));
        }

        let nutrition = estimate_nutrition_from_foods_with_references(
            &input.foods,
            self.food_nutrition_references.clone(),
        )
        .await;
        let now = chrono::Utc::now().to_rfc3339();
        let pending = PendingMealLog {
            id: input.id,
            user_id: input.user_id,
            session_id: input.session_id,
            day_cycle: day_cycle.clone(),
            foods: input.foods,
            nutrition,
            status: PendingMealLogStatus::Proposed,
            created_at: now.clone(),
            updated_at: now,
        };

        self.pending_meals
            .save_pending_meal(&pending)
            .await
            .map_err(AppError::upstream)?;

        Ok(ProposeMealLogResult {
            summary: format!(
                "我识别到一条{}餐记录，请你确认后再保存。",
                day_cycle.as_str()
            ),
            pending,
        })
    }

    pub async fn update_pending_meal(
        &self,
        input: UpdatePendingMealLogCommand,
    ) -> AppResult<PendingMealLog> {
        let mut existing = self
            .pending_meals
            .find_pending_meal(&input.user_id, &input.pending_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("pending meal log".to_string()))?;

        if existing.status != PendingMealLogStatus::Proposed {
            return Err(AppError::validation("pending meal is not editable"));
        }

        self.ensure_day_is_open(&input.user_id, &existing.session_id)
            .await?;

        let day_cycle = DayCycle::parse(&input.day_cycle)?;
        if input.foods.is_empty() {
            return Err(AppError::validation("pending meal contains no food items"));
        }

        let nutrition = estimate_nutrition_from_foods_with_references(
            &input.foods,
            self.food_nutrition_references.clone(),
        )
        .await;

        existing.day_cycle = day_cycle;
        existing.foods = input.foods;
        existing.nutrition = nutrition;
        existing.updated_at = chrono::Utc::now().to_rfc3339();

        self.pending_meals
            .save_pending_meal(&existing)
            .await
            .map_err(AppError::upstream)?;

        Ok(existing)
    }

    pub async fn confirm_meal(&self, input: ConfirmMealLogCommand) -> AppResult<LogMealResult> {
        let existing = self
            .pending_meals
            .find_pending_meal(&input.user_id, &input.pending_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("pending meal log".to_string()))?;

        if existing.status != PendingMealLogStatus::Proposed {
            return Err(AppError::validation("pending meal is not confirmable"));
        }

        self.ensure_day_is_open(&input.user_id, &existing.session_id)
            .await?;

        let result = self
            .handle_meal(LogMealCommand {
                user_id: input.user_id.clone(),
                session_id: existing.session_id.clone(),
                day_cycle: existing.day_cycle.to_string(),
                foods: existing.foods.clone(),
            })
            .await?;

        self.pending_meals
            .delete_pending_meal(&input.user_id, &input.pending_id)
            .await
            .map_err(AppError::upstream)?;

        Ok(result)
    }

    pub async fn reject_meal(&self, input: RejectMealLogCommand) -> AppResult<()> {
        let pending = self
            .pending_meals
            .find_pending_meal(&input.user_id, &input.pending_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("pending meal log".to_string()))?;

        if pending.status != PendingMealLogStatus::Proposed {
            return Err(AppError::validation("pending meal is not rejectable"));
        }

        self.pending_meals
            .delete_pending_meal(&input.user_id, &input.pending_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn list_pending_meals(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> AppResult<Vec<PendingMealLog>> {
        self.pending_meals
            .list_pending_meals(user_id, session_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn finalize_meal_day(
        &self,
        input: FinalizeMealDayCommand,
    ) -> AppResult<FinalizeMealDayResult> {
        if let Some(existing) = self
            .day_finalizations
            .find_finalization(&input.user_id, &input.session_id)
            .await
            .map_err(AppError::upstream)?
        {
            return Ok(FinalizeMealDayResult {
                finalization: existing,
            });
        }

        let finalization = MealDayFinalization {
            user_id: input.user_id,
            session_id: input.session_id,
            finalized_at: chrono::Utc::now().to_rfc3339(),
        };
        self.day_finalizations
            .save_finalization(&finalization)
            .await
            .map_err(AppError::upstream)?;
        Ok(FinalizeMealDayResult { finalization })
    }

    pub async fn find_meal_day_finalization(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> AppResult<Option<MealDayFinalization>> {
        self.day_finalizations
            .find_finalization(user_id, session_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn save_meal_day_summary(
        &self,
        input: SaveMealDaySummaryCommand,
    ) -> AppResult<SaveMealDaySummaryResult> {
        if input.content.trim().is_empty() {
            return Err(AppError::validation("meal day summary content is empty"));
        }

        let finalization = self
            .day_finalizations
            .find_finalization(&input.user_id, &input.session_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::validation("meal day must be finalized before summary"))?;

        let meals = self
            .meals
            .list_meals(&input.user_id, &input.session_id)
            .await
            .map_err(AppError::upstream)?;

        let context = self
            .user_contexts
            .get_context(&input.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user dietary context".to_string()))?;

        let aggregate = aggregate_nutrition(&meals);
        let nutrition_score = score_nutrition(&aggregate);
        let expectation_match_score =
            score_expectation_match(&aggregate, &context.active_expectations);
        let overall_score =
            ((nutrition_score * 0.55) + (expectation_match_score * 0.45)).clamp(0.0, 100.0);
        let metrics_json = serde_json::to_string(&serde_json::json!({
            "meal_count": meals.len(),
            "total_nutrition": aggregate,
            "active_expectations": context
                .active_expectations
                .iter()
                .map(|item| serde_json::json!({
                    "id": item.id.to_string(),
                    "title": item.title,
                    "kind": item.kind.as_str(),
                    "priority": item.priority,
                }))
                .collect::<Vec<_>>(),
            "scores": {
                "nutrition": nutrition_score,
                "expectation_match": expectation_match_score,
                "overall": overall_score,
            }
        }))
        .map_err(AppError::internal)?;

        let now = chrono::Utc::now().to_rfc3339();
        let summary = MealDaySummary {
            user_id: input.user_id,
            session_id: input.session_id,
            content: input.content.trim().to_string(),
            nutrition_score,
            expectation_match_score,
            overall_score,
            metrics_json,
            finalized_at: finalization.finalized_at,
            created_at: now.clone(),
            updated_at: now,
        };

        self.day_summaries
            .save_summary(&summary)
            .await
            .map_err(AppError::upstream)?;

        Ok(SaveMealDaySummaryResult { summary })
    }

    pub async fn find_meal_day_summary(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> AppResult<Option<MealDaySummary>> {
        self.day_summaries
            .find_summary(user_id, session_id)
            .await
            .map_err(AppError::upstream)
    }

    async fn ensure_day_is_open(&self, user_id: &UserId, session_id: &str) -> AppResult<()> {
        if self
            .day_finalizations
            .find_finalization(user_id, session_id)
            .await
            .map_err(AppError::upstream)?
            .is_some()
        {
            return Err(AppError::validation(format!(
                "meal day `{session_id}` has been finalized and no longer accepts meal logs"
            )));
        }
        Ok(())
    }
}

fn aggregate_nutrition(meals: &[MealRecord]) -> Nutrition {
    let mut total = Nutrition::default();
    for meal in meals {
        total.calories += meal.nutrition.calories;
        total.protein_g += meal.nutrition.protein_g;
        total.fat_g += meal.nutrition.fat_g;
        total.carbs_g += meal.nutrition.carbs_g;
        total.fiber_g += meal.nutrition.fiber_g;
        total.sugar_g += meal.nutrition.sugar_g;
        total.saturated_fat_g += meal.nutrition.saturated_fat_g;
        total.sodium_mg += meal.nutrition.sodium_mg;
        total.potassium_mg += meal.nutrition.potassium_mg;
        total.calcium_mg += meal.nutrition.calcium_mg;
        total.iron_mg += meal.nutrition.iron_mg;
        total.magnesium_mg += meal.nutrition.magnesium_mg;
        total.zinc_mg += meal.nutrition.zinc_mg;
        total.vitamin_a_rae_ug += meal.nutrition.vitamin_a_rae_ug;
        total.vitamin_c_mg += meal.nutrition.vitamin_c_mg;
        total.vitamin_d_ug += meal.nutrition.vitamin_d_ug;
        total.vitamin_e_mg += meal.nutrition.vitamin_e_mg;
        total.vitamin_k_ug += meal.nutrition.vitamin_k_ug;
        total.thiamin_b1_mg += meal.nutrition.thiamin_b1_mg;
        total.riboflavin_b2_mg += meal.nutrition.riboflavin_b2_mg;
        total.niacin_b3_mg += meal.nutrition.niacin_b3_mg;
        total.vitamin_b6_mg += meal.nutrition.vitamin_b6_mg;
        total.folate_ug += meal.nutrition.folate_ug;
        total.vitamin_b12_ug += meal.nutrition.vitamin_b12_ug;
    }
    total
}

fn score_nutrition(total: &Nutrition) -> f32 {
    let mut score = 70.0_f32;
    score += range_bonus(total.calories, 1200.0, 2300.0, 14.0);
    score += min_bonus(total.protein_g, 45.0, 10.0);
    score += min_bonus(total.fiber_g, 18.0, 8.0);
    score -= excess_penalty(total.sugar_g, 50.0, 10.0);
    score -= excess_penalty(total.sodium_mg, 2300.0, 10.0);
    score.clamp(0.0, 100.0)
}

fn score_expectation_match(
    total: &Nutrition,
    expectations: &[domain::UserHealthExpectation],
) -> f32 {
    if expectations.is_empty() {
        return 70.0;
    }

    let mut scores = Vec::new();
    for expectation in expectations {
        let score = match &expectation.kind {
            HealthExpectationKind::WeightLoss => {
                70.0 + range_bonus(total.calories, 1100.0, 1900.0, 20.0)
                    + min_bonus(total.protein_g, 50.0, 10.0)
                    - excess_penalty(total.sugar_g, 35.0, 12.0)
            }
            HealthExpectationKind::EnergyBoost => {
                68.0 + min_bonus(total.protein_g, 45.0, 12.0)
                    + min_bonus(total.iron_mg, 8.0, 8.0)
                    + min_bonus(total.vitamin_c_mg, 60.0, 8.0)
                    - excess_penalty(total.sugar_g, 60.0, 8.0)
            }
            HealthExpectationKind::BetterSleep => {
                72.0 + min_bonus(total.magnesium_mg, 250.0, 10.0)
                    - excess_penalty(total.sugar_g, 45.0, 8.0)
                    - excess_penalty(total.sodium_mg, 2300.0, 8.0)
            }
            HealthExpectationKind::BloodSugarControl => {
                70.0 + min_bonus(total.fiber_g, 20.0, 12.0) + min_bonus(total.protein_g, 45.0, 8.0)
                    - excess_penalty(total.sugar_g, 30.0, 18.0)
            }
            HealthExpectationKind::Custom(_) => score_nutrition(total),
        };
        scores.push(score.clamp(0.0, 100.0));
    }

    scores.iter().sum::<f32>() / scores.len() as f32
}

fn min_bonus(value: f32, target: f32, max_bonus: f32) -> f32 {
    if target <= 0.0 {
        return 0.0;
    }
    ((value / target).min(1.0) * max_bonus).max(0.0)
}

fn range_bonus(value: f32, min: f32, max: f32, max_bonus: f32) -> f32 {
    if value >= min && value <= max {
        max_bonus
    } else {
        0.0
    }
}

fn excess_penalty(value: f32, threshold: f32, max_penalty: f32) -> f32 {
    if value <= threshold || threshold <= 0.0 {
        return 0.0;
    }
    (((value - threshold) / threshold).min(1.0) * max_penalty).max(0.0)
}
