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

#[derive(Debug, Clone, toasty::Model)]
pub struct ChatMessageAttachmentRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    #[index]
    pub session_id: String,
    #[index]
    pub message_id: String,
    pub kind: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub data_url: Option<String>,
    pub status: String,
    pub created_at: String,
}
