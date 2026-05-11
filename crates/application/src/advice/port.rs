use async_trait::async_trait;
use domain::UserId;

#[async_trait]
pub trait KnowledgeBasePort: Send + Sync {
    async fn search_user_knowledge(
        &self,
        user_id: &UserId,
        query: &str,
    ) -> Result<Vec<String>, String>;
}
