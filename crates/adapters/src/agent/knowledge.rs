use std::sync::Arc;

use app::common::agent::KnowledgeBasePort;
use domain::UserId;

pub struct KnowledgeRetriever {
    source: Arc<dyn KnowledgeBasePort>,
}

impl KnowledgeRetriever {
    pub fn new(source: Arc<dyn KnowledgeBasePort>) -> Self {
        Self { source }
    }

    pub async fn retrieve(&self, user_id: &UserId, query: &str) -> Vec<String> {
        self.source
            .search_user_knowledge(user_id, query)
            .await
            .unwrap_or_default()
    }
}
