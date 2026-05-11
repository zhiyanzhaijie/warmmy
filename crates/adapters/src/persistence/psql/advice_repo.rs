use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;

use application::advice::KnowledgeBasePort;
use domain::UserId;

#[derive(Clone)]
pub struct PsqlAdviceRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl PsqlAdviceRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KnowledgeBasePort for PsqlAdviceRepo {
    async fn search_user_knowledge(&self, user_id: &UserId, query: &str) -> Result<Vec<String>, String> {
        let _db_guard = self.db.lock().await;
        let _ = json!({
            "user_id": user_id.as_str(),
            "query": query,
            "driver": "toasty-postgresql"
        });
        Ok(Vec::new())
    }
}
