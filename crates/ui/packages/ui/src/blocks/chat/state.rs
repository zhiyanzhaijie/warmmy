use dioxus::prelude::*;

use api::meal;

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub text: String,
    pub is_bot: bool,
    pub is_skeleton: bool,
    pub is_streaming: bool,
    #[serde(default)]
    pub attachments: Vec<ChatMessageAttachment>,
    pub pending_meal: Option<meal::PendingMealLogDTO>,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessageAttachment {
    pub id: String,
    pub kind: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub data_url: Option<String>,
    pub status: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComposerImageAttachment {
    pub id: u64,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub bytes: Vec<u8>,
    pub preview_data_url: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PendingConversationMessage {
    pub session_id: String,
    pub content: String,
    pub started: bool,
}

#[derive(Clone, Copy)]
pub struct ConversationTransitionContext {
    pub pending: Signal<Option<PendingConversationMessage>>,
}

pub static CHAT_MESSAGES: GlobalSignal<Vec<ChatMessage>> = Signal::global(Vec::new);
pub static ACTIVE_SESSION_ID: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CHAT_INPUT: GlobalSignal<String> = Signal::global(String::new);
pub static CHAT_NEXT_ID: GlobalSignal<u64> = Signal::global(|| 1_u64);
pub static CHAT_COMPOSER_ATTACHMENTS: GlobalSignal<Vec<ComposerImageAttachment>> =
    Signal::global(Vec::new);
pub static CHAT_ATTACHMENT_NEXT_ID: GlobalSignal<u64> = Signal::global(|| 1_u64);
