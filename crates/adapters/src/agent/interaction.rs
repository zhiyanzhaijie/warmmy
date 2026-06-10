use std::sync::{Arc, Mutex};

use app::agents::AgentInteractionRequest;

#[derive(Clone, Default)]
pub struct AgentInteractionSink {
    requests: Arc<Mutex<Vec<AgentInteractionRequest>>>,
}

impl AgentInteractionSink {
    pub fn push(&self, request: AgentInteractionRequest) {
        if let Ok(mut requests) = self.requests.lock() {
            requests.push(request);
        }
    }

    pub fn drain(&self) -> Vec<AgentInteractionRequest> {
        self.requests
            .lock()
            .map(|mut requests| requests.drain(..).collect())
            .unwrap_or_default()
    }
}
