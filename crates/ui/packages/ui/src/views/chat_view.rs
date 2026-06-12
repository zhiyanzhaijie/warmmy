use dioxus::prelude::*;

use crate::blocks::ChatBlock;

#[component]
pub fn ChatDetailView(session_id: String) -> Element {
    rsx! {
        main { class: "relative h-full min-h-0 overflow-hidden",
            div { class: "pointer-events-none absolute inset-0 z-0 h-full w-full opacity-20 dark:opacity-20",
                img {
                    src: asset!("/assets/wammy_market.svg"),
                    class: "absolute left-1/2 top-[4%] w-[130%] -translate-x-1/2 object-contain md:top-[-10%] md:w-[95%] lg:top-[-20%] lg:w-[85%] 2xl:w-[75%]",
                    alt: "",
                }
            }
            div { class: "relative z-10 h-full",
                ChatBlock { session_id: Some(session_id) }
            }
        }
    }
}
