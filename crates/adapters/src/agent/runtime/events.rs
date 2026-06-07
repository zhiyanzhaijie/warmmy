use app::conversation::{AgentStatusKind, ConversationStreamEvent};
use serde_json::{to_value, Value};

use crate::agent::interaction::AgentInteractionRequest;
use crate::agent::routing::classifier::AgentRoute;

pub fn initial_route_status(route: AgentRoute, has_images: bool) -> String {
    match route {
        AgentRoute::MealIntake => status_event(
            AgentStatusKind::ReadingInput,
            if has_images {
                "我在整理图片里的用餐线索。"
            } else {
                "我在整理这次用餐线索。"
            },
        ),
        AgentRoute::Chat if has_images => {
            status_event(AgentStatusKind::ReadingInput, "我在认真看看这张图片。")
        }
        AgentRoute::Chat => status_event(AgentStatusKind::Thinking, "我在准备这次对话的上下文。"),
    }
}

pub fn status_event(kind: AgentStatusKind, label: &str) -> String {
    stream_event(ConversationStreamEvent::Status {
        kind,
        label: label.to_string(),
    })
}

pub fn interaction_event(interaction: AgentInteractionRequest) -> String {
    let interaction = to_value(interaction).unwrap_or(Value::Null);
    stream_event(ConversationStreamEvent::InteractionRequested { interaction })
}

pub fn stream_event(event: ConversationStreamEvent) -> String {
    match serde_json::to_string(&event) {
        Ok(line) => line + "\n",
        Err(err) => {
            tracing::warn!(error = %err, "failed to serialize conversation stream event");
            String::new()
        }
    }
}
