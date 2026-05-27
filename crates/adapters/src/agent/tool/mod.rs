use std::sync::Arc;

use app::meal::MealCommandHandler;
use rig::tool::ToolDyn;

use crate::agent::interaction::AgentInteractionSink;
use domain::UserId;

mod meal;

pub fn tools(
    user_id: &UserId,
    session_id: &str,
    meal_command: Arc<MealCommandHandler>,
    interaction_sink: AgentInteractionSink,
) -> Vec<Box<dyn ToolDyn>> {
    meal::tools(user_id, session_id, meal_command, interaction_sink)
}
