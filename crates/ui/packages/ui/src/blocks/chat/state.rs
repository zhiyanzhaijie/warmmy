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
    #[serde(default)]
    pub action: Option<ChatMessageAction>,
    pub pending_meal: Option<meal::PendingMealLogDTO>,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessageAction {
    pub label: String,
    pub route: String,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SessionHistoryLoadPhase {
    #[default]
    InitialLoading,
    Ready,
    LoadingOlder,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SessionHistoryWindow {
    pub total_count: usize,
    pub start_index: usize,
    pub end_index: usize,
    pub items: Vec<Option<ChatMessage>>,
    pub next_before_message_id: Option<String>,
    pub has_more: bool,
    pub phase: SessionHistoryLoadPhase,
    pub initial_bottom_done: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ChatActivityKind {
    Thinking,
    ReadingInput,
    CallingModel,
    UsingTool,
    SavingMemory,
    Persisting,
    WaitingUser,
    Cancelling,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChatActivity {
    pub kind: ChatActivityKind,
    pub label: String,
    pub tool_name: Option<String>,
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

#[derive(Clone, Copy, PartialEq)]
pub struct ChatContext {
    pub messages: Signal<Vec<ChatMessage>>,
    pub session_messages: Signal<HashMap<String, Vec<ChatMessage>>>,
    pub session_history_windows: Signal<HashMap<String, SessionHistoryWindow>>,
    pub session_activities: Signal<HashMap<String, ChatActivity>>,
    pub active_session_id: Signal<Option<String>>,
    pub input: Signal<String>,
    pub next_id: Signal<u64>,
    pub composer_attachments: Signal<Vec<ComposerImageAttachment>>,
    pub attachment_next_id: Signal<u64>,
    pub finalizing_day: Signal<bool>,
}

#[derive(Clone)]
pub struct SendConversationMessage {
    handler: Rc<dyn Fn(String, String, Vec<ComposerImageAttachment>, bool)>,
}

#[derive(Clone)]
pub struct FinalizeConversationDay {
    handler: Rc<dyn Fn(String, String)>,
}

impl FinalizeConversationDay {
    pub fn new(handler: Rc<dyn Fn(String, String)>) -> Self {
        Self { handler }
    }

    pub fn call(&self, user_id: String, session_id: String) {
        (self.handler)(user_id, session_id);
    }
}

impl PartialEq for FinalizeConversationDay {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handler, &other.handler)
    }
}

impl SendConversationMessage {
    pub fn new(handler: Rc<dyn Fn(String, String, Vec<ComposerImageAttachment>, bool)>) -> Self {
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

#[derive(Clone, PartialEq)]
pub struct ChatActionContext {
    pub send_message: SendConversationMessage,
    pub finalize_day: FinalizeConversationDay,
}
