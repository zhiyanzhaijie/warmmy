use std::sync::Arc;

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
) -> Vec<Box<dyn ToolDyn>> {
    meal::tools(user_id, session_id, meal_command, interaction_sink)
}
