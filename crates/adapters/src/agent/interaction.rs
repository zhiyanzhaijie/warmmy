use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentInteractionRequest {
    pub id: String,
    pub kind: String,
    pub payload: Value,
}

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
