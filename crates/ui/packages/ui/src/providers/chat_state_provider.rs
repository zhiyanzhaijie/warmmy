use std::collections::HashMap;

use dioxus::prelude::*;

use crate::blocks::{ChatMessage, ChatContext, ComposerImageAttachment};

#[component]
pub fn ChatStateProvider(chat: ChatContext, children: Element) -> Element {
    use_context_provider(|| chat);

    rsx! {
        {children}
    }
}

pub fn use_chat_context_value() -> ChatContext {
    let messages = use_signal(Vec::<ChatMessage>::new);
    let session_messages = use_signal(HashMap::<String, Vec<ChatMessage>>::new);
    let session_activities = use_signal(HashMap::new);
    let active_session_id = use_signal(|| None);
    let input = use_signal(String::new);
    let next_id = use_signal(|| 1_u64);
    let composer_attachments = use_signal(Vec::<ComposerImageAttachment>::new);
    let attachment_next_id = use_signal(|| 1_u64);
    let finalizing_day = use_signal(|| false);

    ChatContext {
        messages,
        session_messages,
        session_activities,
        active_session_id,
        input,
        next_id,
        composer_attachments,
        attachment_next_id,
        finalizing_day,
    }
}
