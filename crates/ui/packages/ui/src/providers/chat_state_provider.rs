use std::collections::HashMap;

use dioxus::prelude::*;

use crate::blocks::{ChatMessage, ChatStateContext, ComposerImageAttachment};

#[component]
pub fn ChatStateProvider(children: Element) -> Element {
    let messages = use_signal(Vec::<ChatMessage>::new);
    let session_messages = use_signal(HashMap::<String, Vec<ChatMessage>>::new);
    let active_session_id = use_signal(|| None);
    let input = use_signal(String::new);
    let next_id = use_signal(|| 1_u64);
    let composer_attachments = use_signal(Vec::<ComposerImageAttachment>::new);
    let attachment_next_id = use_signal(|| 1_u64);

    use_context_provider(|| ChatStateContext {
        messages,
        session_messages,
        active_session_id,
        input,
        next_id,
        composer_attachments,
        attachment_next_id,
    });

    rsx! {
        {children}
    }
}
