use dioxus::prelude::*;

use crate::blocks::ChatBlock;

#[component]
pub fn ChatView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0",
            ChatBlock {}
        }
    }
}
