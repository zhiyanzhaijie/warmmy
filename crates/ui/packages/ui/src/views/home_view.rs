use dioxus::prelude::*;
use dioxus_icons::lucide::{Cloud, Sparkles};
use crate::components::ui::button::Button;
use crate::components::ui::card::Card;

#[component]
pub fn HomeView() -> Element {
    rsx! {
        div {
            class: "flex flex-col md:flex-row items-center justify-center md:justify-between h-full px-6 md:px-12 lg:px-24",
            div {
                class: "flex-1 flex flex-col items-center md:items-start text-center md:text-left mt-10 md:mt-0",
                h1 {
                    class: "font-doodle text-5xl md:text-6xl lg:text-7xl text-foreground mb-6 tracking-wide leading-tight",
                    "Time to "
                    br { class: "hidden md:block" }
                    "eat smart!"
                }
                p {
                    class: "text-muted-foreground font-medium mb-12 text-lg md:text-xl max-w-md",
                    "Log your meals, get tips, and stay on track with your food buddy."
                }
                div {
                    class: "w-full max-w-[280px] md:max-w-[320px] space-y-4",
                    Link {
                        to: "/chat",
                        class: "block w-full",
                        Button {
                            class: "w-full",
                            "Start Chatting"
                        }
                    }
                }
            }

            div {
                class: "flex-1 flex justify-center items-center mt-12 md:mt-0 order-first md:order-last w-full max-w-[280px] md:max-w-none",
                Card {
                    class: "relative w-full max-w-[280px] md:max-w-[320px] lg:max-w-[400px] aspect-square rounded-[3rem] bg-card border border-border flex items-center justify-center shadow-sm",
                    div {
                        class: "absolute top-10 right-10 text-[#FFAD1A] opacity-50",
                        Cloud { class: "w-24 h-12" }
                    }
                    Sparkles { class: "text-[#FFAD1A] w-32 h-32" }
                }
            }
        }
    }
}
