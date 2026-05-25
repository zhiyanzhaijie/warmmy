use dioxus::prelude::*;

use crate::blocks::MeBlock;

#[component]
pub fn MeView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            MeBlock {}
        }
    }
}
