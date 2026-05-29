use dioxus::prelude::*;

use crate::blocks::WarmmyBlock;

#[component]
pub fn WarmmyView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            WarmmyBlock {}
        }
    }
}
