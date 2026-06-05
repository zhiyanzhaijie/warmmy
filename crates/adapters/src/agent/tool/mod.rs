use std::sync::Arc;

use app::meal::MealCommandHandler;
use rig::tool::ToolDyn;

use crate::agent::interaction::AgentInteractionSink;
use crate::agent::routing::classifier::AgentRoute;
use crate::agent::services::nutrition::curator::NutritionCurator;
use crate::agent::services::nutrition::retriever::NutritionReferenceRetriever;
use crate::agent::services::AgentServiceProgress;
use domain::UserId;

mod meal;

pub mod names {
    pub const PROPOSE_MEAL_LOG: &str = "propose_meal_log";
    pub const CONFIRM_MEAL_LOG: &str = "confirm_meal_log";
    pub const REJECT_MEAL_LOG: &str = "reject_meal_log";
}

pub fn tools(
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn ToolDyn>> {
    meal::tools(
        user_id,
        session_id,
        meal_command,
        interaction_sink,
        nutrition_curator,
        nutrition_retriever,
        progress,
    )
}

pub fn tools_for_route(
    route: AgentRoute,
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn ToolDyn>> {
    match route {
        AgentRoute::MealIntake => {
            tracing::info!(
                agent.route = ?route,
                toolset = "meal_intake",
                tool.count = 3,
                "agent tools selected"
            );
            tools(
                user_id,
                session_id,
                meal_command,
                interaction_sink,
                nutrition_curator,
                nutrition_retriever,
                progress,
            )
        }
        AgentRoute::Chat => {
            tracing::info!(
                agent.route = ?route,
                toolset = "none",
                tool.count = 0,
                "agent tools selected"
            );
            Vec::new()
        }
    }
}

pub fn meal_propose_tools(
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn ToolDyn>> {
    meal::meal_propose_tools(
        user_id,
        session_id,
        meal_command,
        interaction_sink,
        nutrition_curator,
        nutrition_retriever,
        progress,
    )
}

pub fn confirm_meal_log_tools(
    user_id: &UserId,
    meal_command: Arc<MealCommandHandler>,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn ToolDyn>> {
    meal::confirm_meal_log_tools(
        user_id,
        meal_command,
        nutrition_curator,
        nutrition_retriever,
        progress,
    )
}

pub fn reject_meal_log_tools(
    user_id: &UserId,
    meal_command: Arc<MealCommandHandler>,
) -> Vec<Box<dyn ToolDyn>> {
    meal::reject_meal_log_tools(user_id, meal_command)
}
