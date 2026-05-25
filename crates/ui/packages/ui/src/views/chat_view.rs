use dioxus::prelude::*;

use crate::blocks::ChatBlock;

#[component]
pub fn ChatView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            ChatBlock { session_id: None }
        }
    }
}

#[component]
pub fn ChatDetailView(session_id: String) -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            ChatBlock { session_id: Some(session_id) }
        }
    }
}
