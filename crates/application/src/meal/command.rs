use std::sync::Arc;

use async_trait::async_trait;
use domain::{DayCycle, MealAdvice, MealRecord, UserId};

use crate::advice::{impls::build_meal_advice_prompt, KnowledgeBasePort};
use crate::app_error::{AppError, AppResult};
use crate::meal::{LlmPort, MealEventHandler, MealRecordRepositoryPort, SessionMemoryPort};
use crate::nutrition::impls::estimate_nutrition_from_foods;
use crate::user::UserProfileRepositoryPort;

#[derive(Debug, Clone)]
pub enum MealSource {
    Text(String),
    ImageUrl(String),
}

#[derive(Debug, Clone)]
pub struct LogMealCommand {
    pub user_id: UserId,
    pub day_cycle: String,
    pub source: MealSource,
}

#[derive(Debug, Clone)]
pub struct LogMealResult {
    pub meal: MealRecord,
    pub advice: MealAdvice,
}

#[async_trait]
pub trait LogMealUseCase: Send + Sync {
    async fn handle_meal(&self, input: LogMealCommand) -> AppResult<LogMealResult>;
}

#[derive(Clone)]
pub struct MealCommandHandler {
    llm: Arc<dyn LlmPort>,
    memory: Arc<dyn SessionMemoryPort>,
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
    meals: Arc<dyn MealRecordRepositoryPort>,
    knowledge: Arc<dyn KnowledgeBasePort>,
    event_handler: Option<MealEventHandler>,
}

impl MealCommandHandler {
    pub fn new(
        llm: Arc<dyn LlmPort>,
        memory: Arc<dyn SessionMemoryPort>,
        user_profiles: Arc<dyn UserProfileRepositoryPort>,
        meals: Arc<dyn MealRecordRepositoryPort>,
        knowledge: Arc<dyn KnowledgeBasePort>,
    ) -> Self {
        Self {
            llm,
            memory,
            user_profiles,
            meals,
            knowledge,
            event_handler: None,
        }
    }

    pub fn with_event_handler(mut self, event_handler: MealEventHandler) -> Self {
        self.event_handler = Some(event_handler);
        self
    }
}

#[async_trait]
impl LogMealUseCase for MealCommandHandler {
    async fn handle_meal(&self, input: LogMealCommand) -> AppResult<LogMealResult> {
        let day_cycle = DayCycle::parse(&input.day_cycle)?;
        let profile = self
            .user_profiles
            .find_profile(&input.user_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("user profile".to_string()))?;

        let foods = match &input.source {
            MealSource::Text(content) => self
                .llm
                .parse_meal_from_text(content)
                .await
                .map_err(AppError::upstream)?,
            MealSource::ImageUrl(image_url) => self
                .llm
                .parse_meal_from_image(image_url)
                .await
                .map_err(AppError::upstream)?,
        };

        if foods.is_empty() {
            return Err(AppError::validation("parsed meal contains no food items"));
        }

        let nutrition = estimate_nutrition_from_foods(&foods);
        let meal = MealRecord {
            user_id: input.user_id.clone(),
            day_cycle: day_cycle.clone(),
            foods,
            nutrition,
        };

        let recent_dialogue = self
            .memory
            .get_recent_dialogue(&input.user_id)
            .await
            .map_err(AppError::upstream)?;
        let knowledge_hits = self
            .knowledge
            .search_user_knowledge(&input.user_id, "diet advice")
            .await
            .map_err(AppError::upstream)?;
        let prompt = build_meal_advice_prompt(&profile, &meal, &recent_dialogue, &knowledge_hits);

        let generated = self
            .llm
            .generate_advice(&prompt)
            .await
            .map_err(AppError::upstream)?;

        self.meals.save_meal(&meal).await.map_err(AppError::upstream)?;
        self.memory
            .append_dialogue(&input.user_id, generated.clone())
            .await
            .map_err(AppError::upstream)?;

        if let Some(event_handler) = &self.event_handler {
            let event = domain::meal::event::MealRecorded {
                user_id: meal.user_id.clone(),
                day_cycle: meal.day_cycle.clone(),
            };
            event_handler.handle_meal_recorded(&event).await?;
        }

        Ok(LogMealResult {
            meal,
            advice: MealAdvice {
                summary: generated,
                next_meal_suggestion: "下一餐建议补充优质蛋白和蔬菜".to_string(),
                warnings: Vec::new(),
            },
        })
    }
}
