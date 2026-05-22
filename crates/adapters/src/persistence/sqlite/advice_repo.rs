use std::sync::Arc;

use app::common::agent::KnowledgeBasePort;
use async_trait::async_trait;
use domain::UserId;
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SqliteAdviceRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteAdviceRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KnowledgeBasePort for SqliteAdviceRepo {
    async fn search_user_knowledge(
        &self,
        user_id: &UserId,
        query: &str,
    ) -> Result<Vec<String>, String> {
        let _db_guard = self.db.lock().await;
        let _ = json!({
            "user_id": user_id.as_str(),
            "query": query,
            "driver": "toasty-sqlite"
        });
        Ok(Vec::new())
    }
}
