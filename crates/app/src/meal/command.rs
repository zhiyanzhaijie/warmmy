use std::sync::Arc;

use crate::app_error::{AppError, AppResult};
use crate::meal::{MealEventHandler, MealRecordRepositoryPort};
use crate::nutrition::impls::estimate_nutrition_from_foods;
use crate::user::UserDietaryContextQueryHandler;
use domain::{DayCycle, MealAdvice, MealRecord, UserId};

#[derive(Debug, Clone)]
pub struct LogMealCommand {
    pub user_id: UserId,
    pub day_cycle: String,
    pub foods: Vec<domain::FoodItem>,
}

#[derive(Debug, Clone)]
pub struct LogMealResult {
    pub meal: MealRecord,
    pub advice: MealAdvice,
}

#[derive(Clone)]
pub struct MealCommandHandler {
    user_contexts: UserDietaryContextQueryHandler,
    meals: Arc<dyn MealRecordRepositoryPort>,
    event_handler: Option<MealEventHandler>,
}

impl MealCommandHandler {
    pub fn new(
        user_contexts: UserDietaryContextQueryHandler,
        meals: Arc<dyn MealRecordRepositoryPort>,
    ) -> Self {
        Self {
            user_contexts,
            meals,
            event_handler: None,
        }
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

        if input.foods.is_empty() {
            return Err(AppError::validation("meal contains no food items"));
        }

        let nutrition = estimate_nutrition_from_foods(&input.foods);
        let calories = nutrition.calories;
        let meal = MealRecord {
            user_id: input.user_id.clone(),
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

        let advice_summary = format!(
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
            advice: MealAdvice {
                summary: advice_summary,
                next_meal_suggestion: "下一餐建议补充优质蛋白和蔬菜".to_string(),
                warnings: Vec::new(),
            },
        })
    }
}
