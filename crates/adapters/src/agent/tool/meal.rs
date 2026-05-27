use std::sync::Arc;

use app::meal::{
    ConfirmMealLogCommand, MealCommandHandler, ProposeMealLogCommand, RejectMealLogCommand,
};

use crate::agent::interaction::{AgentInteractionRequest, AgentInteractionSink};
use domain::{FoodItem, UserId};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("application: {0}")]
    Application(String),
}

#[derive(Clone)]
pub struct ProposeMealLogTool {
    user_id: UserId,
    session_id: String,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
}

impl ProposeMealLogTool {
    pub fn new(
        user_id: UserId,
        session_id: String,
        meal_command: Arc<MealCommandHandler>,
        interaction_sink: AgentInteractionSink,
    ) -> Self {
        Self {
            user_id,
            session_id,
            meal_command,
            interaction_sink,
        }
    }
}

#[derive(Clone)]
pub struct ConfirmMealLogTool {
    user_id: UserId,
    meal_command: Arc<MealCommandHandler>,
}

impl ConfirmMealLogTool {
    pub fn new(user_id: UserId, meal_command: Arc<MealCommandHandler>) -> Self {
        Self {
            user_id,
            meal_command,
        }
    }
}

#[derive(Clone)]
pub struct RejectMealLogTool {
    user_id: UserId,
    meal_command: Arc<MealCommandHandler>,
}

impl RejectMealLogTool {
    pub fn new(user_id: UserId, meal_command: Arc<MealCommandHandler>) -> Self {
        Self {
            user_id,
            meal_command,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FoodItemArg {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MealLogArgs {
    pub day_cycle: String,
    pub foods: Vec<FoodItemArg>,
}

#[derive(Debug, Serialize)]
pub struct MealLogOutput {
    pub pending_id: String,
    pub status: String,
    pub recorded: bool,
    pub requires_user_confirmation: bool,
    pub summary: String,
    pub interaction: AgentInteractionRequest,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfirmMealLogArgs {
    pub pending_id: String,
}

#[derive(Debug, Serialize)]
pub struct ConfirmMealLogOutput {
    pub status: String,
    pub summary: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemArg>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RejectMealLogArgs {
    pub pending_id: String,
}

#[derive(Debug, Serialize)]
pub struct RejectMealLogOutput {
    pub status: String,
    pub summary: String,
}

impl Tool for ProposeMealLogTool {
    const NAME: &'static str = "propose_meal_log";

    type Error = ToolError;
    type Args = MealLogArgs;
    type Output = MealLogOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Create a meal-log draft that requires user confirmation before final saving. Use when the user clearly says they ate/drank concrete food or asks to create a confirmation/draft/confirmation card for a concrete meal. The draft itself is the confirmation step, so do not ask for another natural-language confirmation before calling this tool. Do not use for greetings, general questions, or nutrition questions without concrete food.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "day_cycle": {
                        "type": "string",
                        "enum": ["breakfast", "lunch", "dinner", "snack"],
                        "description": "The meal period. Infer from user's text; use snack when unclear."
                    },
                    "foods": food_items_schema()
                },
                "required": ["day_cycle", "foods"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let foods = normalize_foods(args.foods)?;
        let pending_id = format!(
            "pml-{}",
            chrono::Utc::now()
                .timestamp_nanos_opt()
                .unwrap_or_else(|| chrono::Utc::now().timestamp_micros())
        );
        let result = self
            .meal_command
            .propose_meal(ProposeMealLogCommand {
                id: domain::PendingMealLogId::new_unchecked(pending_id),
                user_id: self.user_id.clone(),
                session_id: self.session_id.clone(),
                day_cycle: args.day_cycle,
                foods,
            })
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        let foods_payload = result
            .pending
            .foods
            .iter()
            .map(|food| {
                json!({
                    "name": food.name,
                    "quantity": food.quantity,
                    "unit": food.unit,
                })
            })
            .collect::<Vec<_>>();
        let payload = json!({
            "id": result.pending.id.to_string(),
            "day_cycle": result.pending.day_cycle.to_string(),
            "foods": foods_payload,
            "nutrition": result.pending.nutrition,
            "status": "proposed",
        });
        let interaction = AgentInteractionRequest {
            id: result.pending.id.to_string(),
            kind: "meal_log_confirmation".to_string(),
            payload,
        };
        self.interaction_sink.push(interaction.clone());

        let food_names = result
            .pending
            .foods
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>()
            .join("、");

        Ok(MealLogOutput {
            pending_id: result.pending.id.to_string(),
            status: "requires_user_confirmation".to_string(),
            recorded: false,
            requires_user_confirmation: true,
            summary: format!(
                "NOT_SAVED_REQUIRES_USER_CONFIRMATION。{} 只是创建了待确认草稿：{}，估算约 {:.0} kcal。你必须明确告诉用户这还没有写入正式 meal log，等待用户在界面确认；绝不能说已经记录、已保存或已记下。",
                result.summary, food_names, result.pending.nutrition.calories
            ),
            interaction,
        })
    }
}

impl Tool for ConfirmMealLogTool {
    const NAME: &'static str = "confirm_meal_log";

    type Error = ToolError;
    type Args = ConfirmMealLogArgs;
    type Output = ConfirmMealLogOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Confirm a user-approved meal-log draft and save it as the final meal record. Use only after the user explicitly confirms the draft or the application passes a confirmed interaction continuation.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pending_id": { "type": "string", "description": "Pending meal draft id" }
                },
                "required": ["pending_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let result = self
            .meal_command
            .confirm_meal(ConfirmMealLogCommand {
                user_id: self.user_id.clone(),
                pending_id: domain::PendingMealLogId::new_unchecked(args.pending_id),
            })
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        Ok(ConfirmMealLogOutput {
            status: "saved".to_string(),
            summary: result.summary,
            day_cycle: result.meal.day_cycle.to_string(),
            foods: result
                .meal
                .foods
                .into_iter()
                .map(|food| FoodItemArg {
                    name: food.name,
                    quantity: food.quantity,
                    unit: food.unit,
                })
                .collect(),
        })
    }
}

impl Tool for RejectMealLogTool {
    const NAME: &'static str = "reject_meal_log";

    type Error = ToolError;
    type Args = RejectMealLogArgs;
    type Output = RejectMealLogOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Reject and remove a meal-log draft after the user says it should not be recorded."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pending_id": { "type": "string", "description": "Pending meal draft id" }
                },
                "required": ["pending_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.meal_command
            .reject_meal(RejectMealLogCommand {
                user_id: self.user_id.clone(),
                pending_id: domain::PendingMealLogId::new_unchecked(args.pending_id),
            })
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        Ok(RejectMealLogOutput {
            status: "rejected".to_string(),
            summary: "这条待确认用餐记录已取消，没有写入正式 meal log。".to_string(),
        })
    }
}

fn normalize_foods(args: Vec<FoodItemArg>) -> Result<Vec<FoodItem>, ToolError> {
    let foods: Vec<FoodItem> = args
        .into_iter()
        .filter(|f| !f.name.trim().is_empty())
        .map(|f| FoodItem::new(f.name, f.quantity.max(0.0), f.unit))
        .collect();

    if foods.is_empty() {
        return Err(ToolError::Validation("no food items".into()));
    }

    Ok(foods)
}

fn food_items_schema() -> Value {
    json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Food or drink name" },
                "quantity": { "type": "number", "description": "Amount consumed. Use 1 when the user gives a concrete food without quantity." },
                "unit": { "type": "string", "description": "Unit such as g/克/份/碗/杯/个/片/ml. Use 份 when the user gives a concrete food without unit." }
            },
            "required": ["name", "quantity", "unit"]
        },
        "description": "Structured list of foods extracted from user message"
    })
}

pub fn tools(
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
) -> Vec<Box<dyn rig::tool::ToolDyn>> {
    vec![
        Box::new(ProposeMealLogTool::new(
            user_id.clone(),
            session_id.to_string(),
            meal_command.clone(),
            interaction_sink,
        )),
        Box::new(ConfirmMealLogTool::new(
            user_id.clone(),
            meal_command.clone(),
        )),
        Box::new(RejectMealLogTool::new(user_id.clone(), meal_command)),
    ]
}
