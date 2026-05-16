use std::sync::Arc;

use async_trait::async_trait;
use crate::advice::impls::build_meal_advice_prompt;
use crate::app_error::{AppError, AppResult};
use crate::common::agent::{
    GuardrailDecision, GuardrailsPort, KnowledgeBasePort, PerceptionInput, PerceptionPort,
    PlanningPort, ReasoningPort, SessionMemoryPort,
};
use crate::meal::impls::parse_food_items_from_perception;
use crate::meal::{MealEventHandler, MealRecordRepositoryPort};
use crate::nutrition::impls::estimate_nutrition_from_foods;
use crate::user::UserProfileRepositoryPort;
use domain::{DayCycle, MealAdvice, MealRecord, UserId};

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
    reasoning: Arc<dyn ReasoningPort>,
    perception: Arc<dyn PerceptionPort>,
    planning: Arc<dyn PlanningPort>,
    guardrails: Arc<dyn GuardrailsPort>,
    memory: Arc<dyn SessionMemoryPort>,
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
    meals: Arc<dyn MealRecordRepositoryPort>,
    knowledge: Arc<dyn KnowledgeBasePort>,
    event_handler: Option<MealEventHandler>,
}

impl MealCommandHandler {
    pub fn new(
        reasoning: Arc<dyn ReasoningPort>,
        perception: Arc<dyn PerceptionPort>,
        planning: Arc<dyn PlanningPort>,
        guardrails: Arc<dyn GuardrailsPort>,
        memory: Arc<dyn SessionMemoryPort>,
        user_profiles: Arc<dyn UserProfileRepositoryPort>,
        meals: Arc<dyn MealRecordRepositoryPort>,
        knowledge: Arc<dyn KnowledgeBasePort>,
    ) -> Self {
        Self {
            reasoning,
            perception,
            planning,
            guardrails,
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

    async fn ensure_allowed(
        &self,
        decision: Result<GuardrailDecision, String>,
        phase: &str,
    ) -> AppResult<()> {
        match decision.map_err(AppError::upstream)? {
            GuardrailDecision::Allow => Ok(()),
            GuardrailDecision::Reject(reason) => {
                Err(AppError::validation(format!("guardrails rejected {phase}: {reason}")))
            }
        }
    }
}

#[async_trait]
impl LogMealUseCase for MealCommandHandler {
    async fn handle_meal(&self, input: LogMealCommand) -> AppResult<LogMealResult> {
        let raw_user_input = match &input.source {
            MealSource::Text(content) => content.as_str(),
            MealSource::ImageUrl(image_url) => image_url.as_str(),
        };
        self.ensure_allowed(self.guardrails.check_input(raw_user_input).await, "input")
            .await?;
        let day_cycle = DayCycle::parse(&input.day_cycle)?;
        let profile = self
            .user_profiles
            .find_profile(&input.user_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("user profile".to_string()))?;

        let instruction = "请提取食物条目并返回 JSON 数组，每个元素格式为: {\"name\": string, \"quantity\": number, \"unit\": string}。如果数量未知，quantity=1.0，unit=\"份\"。";
        let perception_output = match &input.source {
            MealSource::Text(content) => {
                self.perception
                    .perceive(PerceptionInput::Text(content.clone()), instruction)
                    .await
                    .map_err(AppError::upstream)?
            }
            MealSource::ImageUrl(image_url) => {
                self.perception
                    .perceive(PerceptionInput::ImageUrl(image_url.clone()), instruction)
                    .await
                    .map_err(AppError::upstream)?
            }
        };
        let foods = parse_food_items_from_perception(&perception_output).map_err(AppError::upstream)?;

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
        let plan_steps = self
            .planning
            .plan(
                "为用户生成下一餐建议",
                &[recent_dialogue.join(" | "), knowledge_hits.join(" | ")],
            )
            .await
            .map_err(AppError::upstream)?;
        let prompt = format!(
            "{}\nPlanning steps: {}",
            build_meal_advice_prompt(&profile, &meal, &recent_dialogue, &knowledge_hits),
            plan_steps.join(" -> ")
        );

        let generated = self
            .reasoning
            .complete_text("你是专业营养顾问，请提供简洁、可执行的饮食建议。", &prompt)
            .await
            .map_err(AppError::upstream)?;
        let effect = format!(
            "save_meal_and_append_dialogue:user_id={},day_cycle={}",
            input.user_id.as_str(),
            day_cycle.as_str()
        );
        self.ensure_allowed(self.guardrails.check_effect(&effect).await, "effect")
            .await?;
        self.ensure_allowed(self.guardrails.check_output(&generated).await, "output")
            .await?;

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
