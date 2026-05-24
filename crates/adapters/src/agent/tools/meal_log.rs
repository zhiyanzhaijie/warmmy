use std::sync::Arc;

use app::meal::{LogMealCommand, MealCommandHandler};
use domain::{FoodItem, UserId};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

use serde_json::json;

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
}

impl MealLogTool {
    pub fn new(user_id: UserId, meal_command: Arc<MealCommandHandler>) -> Self {
        Self {
            user_id,
            meal_command,
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
            description: "Record a user's meal only when the user clearly says they ate or drank something. Do not use this tool for greetings, general chat, questions, or nutrition advice without a concrete meal to log.".to_string(),
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

        Ok(MealLogOutput {
            recorded: true,
            summary: format!(
                "{}。已记录：{}，约 {:.0} kcal",
                result.advice.summary, food_names, result.meal.nutrition.calories
            ),
        })
    }
}
