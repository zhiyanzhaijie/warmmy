#[derive(Debug, Clone, toasty::Model)]
pub struct ChatMessageRow {
    #[key]
    #[auto]
    pub id: i32,
    #[index]
    pub user_id: String,
    #[index]
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}
