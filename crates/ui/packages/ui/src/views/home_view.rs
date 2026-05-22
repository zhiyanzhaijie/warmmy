use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::input::Input;
use crate::blocks::ChatBlock;
use crate::blocks::CHAT_MESSAGES;
use crate::views::FIRST_MSG;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Cloud, Send, Sparkles};
use chrono::Utc;

#[component]
pub fn HomeView() -> Element {
    let mut input = use_signal(String::new);

    let mut start_chat_with_msg = move || {
        let content = input().trim().to_string();
        if content.is_empty() {
            return;
        }

        input.set(String::new());

        // Set FIRST_MSG to let ChatBlock initiate the stream
        *FIRST_MSG.write() = Some(content);

        // Transition to detail URL of today's date
        let today_date = Utc::now().format("%Y-%m-%d").to_string();
        navigator().push(format!("/{}", today_date));
    };

    if !CHAT_MESSAGES().is_empty() {
        rsx! {
            ChatBlock { session_id: None }
        }
    } else {
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
                        class: "text-muted-foreground font-medium mb-8 text-lg md:text-xl max-w-md",
                        "Log your meals, get tips, and stay on track with your food buddy."
                    }
                    
                    div {
                        class: "w-full max-w-md md:max-w-lg mb-8",
                        div {
                            class: "relative flex items-center bg-card rounded-[28px] border border-border/80 shadow-md p-2 focus-within:ring-2 ring-[#FFAD1A]/30 transition-all",
                            Input {
                                class: "flex-1 bg-transparent px-5 py-3 outline-none font-medium text-foreground placeholder:text-muted-foreground border-none shadow-none text-base",
                                r#type: "text",
                                placeholder: "我今天中午吃了一碗牛肉面...",
                                value: input(),
                                oninput: move |e: FormEvent| input.set(e.value()),
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        start_chat_with_msg();
                                    }
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Icon,
                                class: "bg-[#FFAD1A] hover:bg-[#FFAD1A]/90 p-3.5 rounded-full text-white transition-all shadow-sm",
                                onclick: move |_| start_chat_with_msg(),
                                Send { size: 22, class: "ml-0.5" }
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
}
