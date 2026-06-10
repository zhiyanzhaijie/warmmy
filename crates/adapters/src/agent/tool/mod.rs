use std::sync::Arc;

use app::agents::{
    AgentRoute, AgentServiceProgress, AgentToolId, NutritionCurator, NutritionReferenceRetriever,
    RouteRegistry,
};
use app::meal::MealCommandHandler;
use rig::tool::ToolDyn;

use crate::agent::interaction::AgentInteractionSink;
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
    let tool_ids = RouteRegistry::new().tool_ids(route);
    let tools = tools_for_ids(
        tool_ids,
        user_id,
        session_id,
        meal_command,
        interaction_sink,
        nutrition_curator,
        nutrition_retriever,
        progress,
    );
    tracing::info!(
        agent.route = ?route,
        tool.count = tools.len(),
        "agent tools selected"
    );
    tools
}

pub fn tools_for_ids(
    tool_ids: &[AgentToolId],
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
    nutrition_curator: Option<Arc<dyn NutritionCurator>>,
    nutrition_retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
) -> Vec<Box<dyn ToolDyn>> {
    let mut tools = Vec::new();
    for tool_id in tool_ids {
        match tool_id {
            AgentToolId::ProposeMealLog => tools.extend(meal::meal_propose_tools(
                user_id,
                session_id,
                meal_command.clone(),
                interaction_sink.clone(),
                nutrition_curator.clone(),
                nutrition_retriever.clone(),
                progress.clone(),
            )),
            AgentToolId::ConfirmMealLog => tools.extend(meal::confirm_meal_log_tools(
                user_id,
                meal_command.clone(),
                nutrition_curator.clone(),
                nutrition_retriever.clone(),
                progress.clone(),
            )),
            AgentToolId::RejectMealLog => {
                tools.extend(meal::reject_meal_log_tools(user_id, meal_command.clone()))
            }
        }
    }
    tools
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
