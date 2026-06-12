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
                class: "absolute left-1/2 top-[6%] w-[135%] -translate-x-1/2 object-contain md:top-[-8%] md:w-[100%] lg:top-[-18%] lg:w-[88%] lg:[-webkit-mask-image:linear-gradient(to_right,transparent_0%,black_24%,black_76%,transparent_100%)] lg:[mask-image:linear-gradient(to_right,transparent_0%,black_24%,black_76%,transparent_100%)] 2xl:w-[78%]",
                alt: "",
            }
        }
    }
}
