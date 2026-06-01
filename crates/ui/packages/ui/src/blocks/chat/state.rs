use dioxus::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

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

#[derive(Clone, Copy)]
pub struct ChatStateContext {
    pub messages: Signal<Vec<ChatMessage>>,
    pub session_messages: Signal<HashMap<String, Vec<ChatMessage>>>,
    pub active_session_id: Signal<Option<String>>,
    pub input: Signal<String>,
    pub next_id: Signal<u64>,
    pub composer_attachments: Signal<Vec<ComposerImageAttachment>>,
    pub attachment_next_id: Signal<u64>,
}

#[derive(Clone)]
pub struct SendConversationMessage {
    handler: Rc<dyn Fn(String, String, Vec<ComposerImageAttachment>, bool)>,
}

impl SendConversationMessage {
    pub fn new(
        handler: Rc<dyn Fn(String, String, Vec<ComposerImageAttachment>, bool)>,
    ) -> Self {
        Self { handler }
    }

    pub fn call(
        &self,
        session_id: String,
        content: String,
        attachments: Vec<ComposerImageAttachment>,
        route_after_stream: bool,
    ) {
        (self.handler)(session_id, content, attachments, route_after_stream);
    }
}

impl PartialEq for SendConversationMessage {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handler, &other.handler)
    }
}

#[derive(Clone, Copy)]
pub struct ChatRuntimeContext {
    pub send_message: Signal<Option<SendConversationMessage>>,
}
