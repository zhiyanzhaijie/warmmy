use dioxus::prelude::*;

use crate::blocks::{TravelDetailBlock, TravelListBlock};

#[component]
pub fn TravelView() -> Element {
    rsx! {
        main { class: "relative h-full min-h-0 overflow-hidden",
            TravelBackground {}
            div { class: "relative z-10 h-full",
                TravelListBlock {}
            }
        }
    }
}

#[component]
pub fn TravelDetailView(summary_id: String) -> Element {
    rsx! {
        main { class: "relative h-full min-h-0 overflow-hidden",
            TravelBackground {}
            div { class: "relative z-10 h-full",
                TravelDetailBlock { summary_id }
            }
        }
    }
}

#[component]
fn TravelBackground() -> Element {
    rsx! {
        div { class: "pointer-events-none absolute inset-0 z-0 h-full w-full opacity-60 dark:opacity-40",
            img {
                src: asset!("/assets/wammy_travel.svg"),
                class: "absolute left-[50%] top-[6%] w-[135%] -translate-x-[50%] object-contain md:top-[-8%] md:w-[100%] lg:left-[auto] lg:right-[-26%] lg:top-[-18%] lg:w-[88%] lg:-translate-x-0 2xl:right-[-20%] 2xl:w-[78%]",
                alt: "",
            }
        }
    }
}
