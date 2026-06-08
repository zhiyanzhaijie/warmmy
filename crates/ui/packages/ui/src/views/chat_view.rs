use dioxus::prelude::*;

use crate::blocks::ChatBlock;

#[component]
pub fn ChatView() -> Element {
    rsx! {
        main { class: "relative h-full min-h-0 overflow-hidden",
            div { class: "pointer-events-none absolute inset-0 z-0 h-full w-full opacity-60 dark:opacity-40",
                img {
                    src: asset!("/assets/wammy_market.svg"),
                    class: "absolute left-[50%] top-[4%] w-[130%] -translate-x-[50%] object-contain md:top-[-10%] md:w-[95%] lg:left-[auto] lg:right-[-25%] lg:top-[-20%] lg:w-[85%] lg:-translate-x-0 2xl:right-[-20%] 2xl:w-[75%]",
                    alt: "",
                }
            }
            div { class: "relative z-10 h-full",
                ChatBlock { session_id: None }
            }
        }
    }
}

#[component]
pub fn ChatDetailView(session_id: String) -> Element {
    rsx! {
        main { class: "relative h-full min-h-0 overflow-hidden",
            div { class: "pointer-events-none absolute inset-0 z-0 h-full w-full opacity-20 dark:opacity-20",
                img {
                    src: asset!("/assets/wammy_market.svg"),
                    class: "absolute left-[50%] top-[4%] w-[130%] -translate-x-[50%] object-contain md:top-[-10%] md:w-[95%] lg:left-[auto] lg:right-[-25%] lg:top-[-20%] lg:w-[85%] lg:-translate-x-0 2xl:right-[-20%] 2xl:w-[75%]",
                    alt: "",
                }
            }
            div { class: "relative z-10 h-full",
                ChatBlock { session_id: Some(session_id) }
            }
        }
    }
}
