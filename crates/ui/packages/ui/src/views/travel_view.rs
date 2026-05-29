use dioxus::prelude::*;

use crate::blocks::{TravelDetailBlock, TravelListBlock};

#[component]
pub fn TravelView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            TravelListBlock {}
        }
    }
}

#[component]
pub fn TravelDetailView(summary_id: String) -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            TravelDetailBlock { summary_id }
        }
    }
}
