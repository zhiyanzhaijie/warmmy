use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use application::common::agent::SessionMemoryPort;
use async_trait::async_trait;
use domain::UserId;

#[derive(Clone, Default)]
pub struct LocalMemory {
    session_messages: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

#[async_trait]
impl SessionMemoryPort for LocalMemory {
    async fn get_recent_dialogue(&self, user_id: &UserId) -> Result<Vec<String>, String> {
        let store = self
            .session_messages
            .lock()
            .map_err(|_| "memory lock poisoned".to_string())?;
        Ok(store.get(user_id.as_str()).cloned().unwrap_or_default())
    }

    async fn append_dialogue(&self, user_id: &UserId, message: String) -> Result<(), String> {
        let mut store = self
            .session_messages
            .lock()
            .map_err(|_| "memory lock poisoned".to_string())?;
        store
            .entry(user_id.as_str().to_string())
            .or_default()
            .push(message);
        Ok(())
    }
}
