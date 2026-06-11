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
