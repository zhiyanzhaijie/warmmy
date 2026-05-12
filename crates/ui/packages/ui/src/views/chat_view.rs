use dioxus::prelude::*;

use crate::blocks::ChatBlock;

#[component]
pub fn ChatView() -> Element {
    rsx! {
        main {
            ChatBlock {}
        }
    }
}
