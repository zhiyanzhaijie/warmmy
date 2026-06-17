#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EchoResponse {
    pub reply: String,
    pub session_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct ChatSendInput {
    pub text: String,
    #[serde(default)]
    pub attachments: Vec<ChatImageAttachmentInput>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChatImageAttachmentInput {
    pub asset_id: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub preview_data_url: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct SessionHistoryCursorInput {
    pub limit: Option<u32>,
    pub before_message_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct SessionHistoryCursorPage {
    pub items: Vec<app::conversation::ChatMessage>,
    pub next_before_message_id: Option<String>,
    pub has_more: bool,
    pub total_count: usize,
    pub start_index: usize,
    pub end_index: usize,
}
