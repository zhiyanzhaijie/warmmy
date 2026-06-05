use std::sync::Arc;

use app::meal::{
    ConfirmMealLogCommand, MealCommandHandler, RejectMealLogCommand, UpdatePendingMealLogCommand,
};

use crate::agent::chains::meal_intake::{MealIntakeChain, MealIntakeFood, MealIntakeInput};
use crate::agent::interaction::{AgentInteractionRequest, AgentInteractionSink};
use crate::agent::services::nutrition::curator::NutritionCurator;
use crate::agent::services::nutrition::estimator::MealNutritionEstimator;
use crate::agent::services::nutrition::retriever::NutritionReferenceRetriever;
use crate::agent::services::AgentServiceProgress;
use crate::agent::tool::names;
use domain::UserId;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("application: {0}")]
    Application(String),
}

#[derive(Clone)]
pub struct ProposeMealLogTool {
    user_id: UserId,
    session_id: String,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    progress: Option<Arc<dyn AgentServiceProgress>>,
}

impl ProposeMealLogTool {
    pub fn new(
        user_id: UserId,
        session_id: String,
        meal_command: Arc<MealCommandHandler>,
        interaction_sink: AgentInteractionSink,
        progress: Option<Arc<dyn AgentServiceProgress>>,
    ) -> Self {
        Self {
            user_id,
            session_id,
            meal_command,
            interaction_sink,
            progress,
        }
    }
}

#[derive(Clone)]
pub struct ConfirmMealLogTool {
    user_id: UserId,
    meal_command: Arc<MealCommandHandler>,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
}

impl ConfirmMealLogTool {
    pub fn new(
        user_id: UserId,
        meal_command: Arc<MealCommandHandler>,
        nutrition_curator: Option<Arc<dyn NutritionCurator>>,
        nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
        progress: Option<Arc<dyn AgentServiceProgress>>,
    ) -> Self {
        Self {
            user_id,
            meal_command,
            nutrition_curator,
            nutrition_retriever,
            progress,
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
    pub grams: f32,
    #[serde(default)]
    pub amount_confidence: Option<f32>,
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
    const NAME: &'static str = names::PROPOSE_MEAL_LOG;

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
        tracing::info!(
            tool.name = Self::NAME,
            session.id = %self.session_id,
            day_cycle = %args.day_cycle,
            food.count = args.foods.len(),
            "propose_meal_log tool called"
        );
        let mut chain = MealIntakeChain::new(self.meal_command.clone());
        if let Some(progress) = &self.progress {
            chain = chain.with_progress(progress.clone());
        }
        let output = match chain
            .propose(MealIntakeInput {
                user_id: self.user_id.clone(),
                session_id: self.session_id.clone(),
                day_cycle: args.day_cycle,
                foods: args
                    .foods
                    .into_iter()
                    .map(|food| MealIntakeFood {
                        name: food.name,
                        grams: food.grams,
                        amount_confidence: food.amount_confidence,
                    })
                    .collect(),
            })
            .await
        {
            Ok(output) => output,
            Err(err) => {
                tracing::warn!(
                    tool.name = Self::NAME,
                    session.id = %self.session_id,
                    error = %err,
                    "propose_meal_log tool failed"
                );
                return Err(ToolError::Application(err.to_string()));
            }
        };
        let result = output.result;
        tracing::info!(
            tool.name = Self::NAME,
            session.id = %self.session_id,
            pending.id = %result.pending.id,
            food.count = result.pending.foods.len(),
            "pending meal proposed"
        );
        let foods_payload = result
            .pending
            .foods
            .iter()
            .map(|food| {
                json!({
                    "name": food.name,
                    "quantity": food.quantity,
                    "unit": food.unit,
                    "estimated_grams": food.estimated_grams,
                    "amount_confidence": food.amount_confidence,
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
        tracing::info!(
            tool.name = Self::NAME,
            session.id = %self.session_id,
            interaction.id = %interaction.id,
            interaction.kind = %interaction.kind,
            "agent interaction emitted"
        );

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
            summary: meal_log_summary(&result.summary, &food_names),
            interaction,
        })
    }
}

impl Tool for ConfirmMealLogTool {
    const NAME: &'static str = names::CONFIRM_MEAL_LOG;

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
        let pending_id = domain::PendingMealLogId::new_unchecked(args.pending_id.clone());
        let pending = self
            .meal_command
            .find_pending_meal(&self.user_id, &pending_id)
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        let mut nutrition_estimator = MealNutritionEstimator::new();
        if let Some(reference_repo) = self.meal_command.food_nutrition_references() {
            nutrition_estimator = nutrition_estimator.with_reference_repo(reference_repo);
        }
        if let Some(nutrition_curator) = &self.nutrition_curator {
            nutrition_estimator = nutrition_estimator.with_curator(nutrition_curator.clone());
        }
        if let Some(nutrition_retriever) = &self.nutrition_retriever {
            nutrition_estimator = nutrition_estimator.with_retriever(nutrition_retriever.clone());
        }
        if let Some(progress) = &self.progress {
            nutrition_estimator = nutrition_estimator.with_progress(progress.clone());
        }
        let nutrition_output = nutrition_estimator
            .estimate(&pending.foods)
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        self.meal_command
            .update_pending_meal(UpdatePendingMealLogCommand {
                user_id: self.user_id.clone(),
                pending_id: pending_id.clone(),
                day_cycle: pending.day_cycle.to_string(),
                foods: pending.foods.clone(),
                nutrition: Some(nutrition_output.nutrition),
            })
            .await
            .map_err(|err| ToolError::Application(err.to_string()))?;

        let result = self
            .meal_command
            .confirm_meal(ConfirmMealLogCommand {
                user_id: self.user_id.clone(),
                pending_id,
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
                    grams: food.estimated_grams.unwrap_or_else(|| {
                        if matches!(food.unit.as_str(), "g" | "克") {
                            food.quantity
                        } else {
                            0.0
                        }
                    }),
                    amount_confidence: food.amount_confidence,
                })
                .collect(),
        })
    }
}

impl Tool for RejectMealLogTool {
    const NAME: &'static str = names::REJECT_MEAL_LOG;

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

fn meal_log_summary(base_summary: &str, food_names: &str) -> String {
    format!(
        "NOT_SAVED_REQUIRES_USER_CONFIRMATION。{} 只是创建了待确认草稿：{}。你必须明确告诉用户这还没有写入正式 meal log，等待用户在界面确认；绝不能说已经记录、已保存或已记下。营养成分会在用户确认最终食物和分量后再计算。",
        base_summary, food_names
    )
}

fn food_items_schema() -> Value {
    json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Food or drink name" },
                "grams": { "type": "number", "description": "Estimated edible weight in grams. Convert vague portions such as one serving, one bowl, one cup, one piece, or image-estimated portions into grams before calling this tool." },
                "amount_confidence": { "type": ["number", "null"], "description": "Confidence for grams from 0 to 1. Use lower confidence for image-only or vague portions." }
            },
            "required": ["name", "grams"]
        },
        "description": "Structured list of foods extracted from user message"
    })
}

pub fn tools(
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn rig::tool::ToolDyn>> {
    vec![
        Box::new(ProposeMealLogTool::new(
            user_id.clone(),
            session_id.to_string(),
            meal_command.clone(),
            interaction_sink,
            progress.clone(),
        )),
        Box::new(ConfirmMealLogTool::new(
            user_id.clone(),
            meal_command.clone(),
            nutrition_curator,
            nutrition_retriever,
            progress,
        )),
        Box::new(RejectMealLogTool::new(user_id.clone(), meal_command)),
    ]
}

pub fn meal_propose_tools(
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    _nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    _nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn rig::tool::ToolDyn>> {
    vec![Box::new(ProposeMealLogTool::new(
        user_id.clone(),
        session_id.to_string(),
        meal_command,
        interaction_sink,
        progress,
    ))]
}

pub fn confirm_meal_log_tools(
    user_id: &UserId,
    meal_command: Arc<MealCommandHandler>,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn rig::tool::ToolDyn>> {
    vec![Box::new(ConfirmMealLogTool::new(
        user_id.clone(),
        meal_command,
        nutrition_curator,
        nutrition_retriever,
        progress,
    ))]
}

pub fn reject_meal_log_tools(
    user_id: &UserId,
    meal_command: Arc<MealCommandHandler>,
) -> Vec<Box<dyn rig::tool::ToolDyn>> {
    vec![Box::new(RejectMealLogTool::new(
        user_id.clone(),
        meal_command,
    ))]
}
