use std::sync::Arc;

use app::meal::{LogMealCommand, MealCommandHandler};
use domain::{FoodItem, UserId};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

use serde_json::json;

use crate::agent::memory::long_term::rag::{embed_rag_document, RagConfig, RagDocument};

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("repository: {0}")]
    Repository(String),
    #[error("domain: {0}")]
    Domain(String),
    #[error("application: {0}")]
    Application(String),
}

#[derive(Clone)]
pub struct MealLogTool {
    user_id: UserId,
    meal_command: Arc<MealCommandHandler>,
    rag_config: RagConfig,
}

impl MealLogTool {
    pub fn new(
        user_id: UserId,
        meal_command: Arc<MealCommandHandler>,
        rag_config: RagConfig,
    ) -> Self {
        Self {
            user_id,
            meal_command,
            rag_config,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FoodItemArg {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct MealLogArgs {
    pub day_cycle: String,
    pub foods: Vec<FoodItemArg>,
}

#[derive(Debug, Serialize)]
pub struct MealLogOutput {
    pub recorded: bool,
    pub summary: String,
}

impl Tool for MealLogTool {
    const NAME: &'static str = "meal_log";

    type Error = ToolError;
    type Args = MealLogArgs;
    type Output = MealLogOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Record a user's meal only when the user clearly says they ate or drank something. Do not use this tool for greetings, general chat, questions, or nutrition questions without a concrete meal to log.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "day_cycle": {
                        "type": "string",
                        "enum": ["breakfast", "lunch", "dinner", "snack"],
                        "description": "The meal period. Infer from user's text; use snack when unclear."
                    },
                    "foods": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Food or drink name" },
                                "quantity": { "type": "number", "description": "Amount consumed, default 1" },
                                "unit": { "type": "string", "description": "Unit such as 份/碗/杯/个/片, default 份" }
                            },
                            "required": ["name", "quantity", "unit"]
                        },
                        "description": "Structured list of foods extracted from user message"
                    }
                },
                "required": ["day_cycle", "foods"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let foods: Vec<FoodItem> = args
            .foods
            .into_iter()
            .filter(|f| !f.name.trim().is_empty())
            .map(|f| FoodItem::new(f.name, f.quantity.max(0.0), f.unit))
            .collect();

        if foods.is_empty() {
            return Err(ToolError::Validation("no food items".into()));
        }

        let result = self
            .meal_command
            .handle_meal(LogMealCommand {
                user_id: self.user_id.clone(),
                day_cycle: args.day_cycle,
                foods,
            })
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        let food_names = result
            .meal
            .foods
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>()
            .join("、");

        if let Err(err) = self.embed_meal_memory(&result).await {
            tracing::warn!(error = %err, "failed to embed meal log into rag memory");
        }

        Ok(MealLogOutput {
            recorded: true,
            summary: format!(
                "{}。已记录：{}，约 {:.0} kcal",
                result.summary, food_names, result.meal.nutrition.calories
            ),
        })
    }
}

impl MealLogTool {
    async fn embed_meal_memory(&self, result: &app::meal::LogMealResult) -> Result<(), ToolError> {
        let meal = &result.meal;
        let foods = meal
            .foods
            .iter()
            .map(|food| format!("{}{}{}", food.name, food.quantity, food.unit))
            .collect::<Vec<_>>()
            .join("、");
        let now = chrono::Utc::now();
        let document = RagDocument {
            id: format!(
                "meal:{}:{}",
                self.user_id.as_str(),
                now.timestamp_nanos_opt().unwrap_or_else(|| now.timestamp_micros())
            ),
            content: format!(
                "用户{}记录{}餐：{}。估算营养：约{:.0}kcal，蛋白质约{:.1}g，碳水约{:.1}g，脂肪约{:.1}g。{}",
                self.user_id.as_str(),
                meal.day_cycle.as_str(),
                foods,
                meal.nutrition.calories,
                meal.nutrition.protein_g,
                meal.nutrition.carbs_g,
                meal.nutrition.fat_g,
                result.summary
            ),
            source: "meal_log".to_string(),
        };

        embed_rag_document(&self.rag_config, document)
            .await
            .map_err(|err| ToolError::Application(err.to_string()))
    }
}
