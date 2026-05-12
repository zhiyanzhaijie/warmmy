use dioxus::prelude::*;
use dioxus_icons::lucide::{Send, Sparkles};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::input::Input;

#[derive(Clone, PartialEq)]
struct ChatMessage {
    id: u64,
    text: String,
    is_bot: bool,
}

#[component]
pub fn ChatBlock() -> Element {
    let mut messages = use_signal(|| {
        vec![
            ChatMessage {
                id: 1,
                text: "Hey there! Ready to track some meals today? 🍎".to_string(),
                is_bot: true,
            },
            ChatMessage {
                id: 2,
                text: "I just had a bagel and coffee for breakfast.".to_string(),
                is_bot: false,
            },
            ChatMessage {
                id: 3,
                text: "Got it! That's roughly 420 kcal. I've logged it to your plan.".to_string(),
                is_bot: true,
            },
        ]
    });
    let mut input = use_signal(String::new);
    let mut next_id = use_signal(|| 4_u64);

    let mut send_message = move || {
        let content = input().trim().to_string();
        if content.is_empty() {
            return;
        }

        let user_id = next_id();
        next_id += 1;
        messages.write().push(ChatMessage {
            id: user_id,
            text: content,
            is_bot: false,
        });
        input.set(String::new());

        let bot_id = next_id();
        next_id += 1;
        messages.write().push(ChatMessage {
            id: bot_id,
            text: "I'm just a doodle demo, but that sounds delicious! 😋".to_string(),
            is_bot: true,
        });
    };
    rsx! {
        Card {
            class: "flex flex-col h-full min-h-0 max-w-3xl mx-auto border-x border-border/50 bg-card/30",

            div {
                class: "flex items-center py-4 px-6 border-b border-border sticky top-0 bg-background/80 backdrop-blur z-10",
                h2 {
                    class: "font-doodle text-2xl font-bold flex items-center gap-2 text-foreground",
                    Sparkles { size: 24, class: "text-[#FFAD1A]" }
                    "Diet Buddy"
                }
            }

            div {
                class: "flex-1 min-h-0 overflow-y-auto space-y-6 pt-6 pb-6 px-6 hide-scrollbar",
                for msg in messages().iter() {
                    div {
                        key: "{msg.id}",
                        class: "max-w-[80%] md:max-w-[70%] rounded-[24px] p-4 font-medium text-[15px] leading-relaxed shadow-sm",
                        class: if msg.is_bot {
                            "bg-muted border border-border text-foreground self-start rounded-tl-sm"
                        } else {
                            "bg-foreground text-background self-end ml-auto rounded-tr-sm"
                        },
                        "{msg.text}"
                    }
                }
            }

            div {
                class: "w-full px-6 pt-4 pb-6 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80",
                div {
                    class: "relative flex items-center bg-card rounded-[24px] border border-border shadow-sm p-1.5 focus-within:ring-2 ring-[#FFAD1A]/20 transition-all",
                    Input {
                        class: "flex-1 bg-transparent px-4 py-2 outline-none font-medium text-foreground placeholder:text-muted-foreground border-none shadow-none",
                        r#type: "text",
                        placeholder: "Type your meal...",
                        value: input(),
                        oninput: move |e: FormEvent| input.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                send_message();
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        class: "bg-foreground p-3 rounded-full text-background hover:opacity-90 transition-opacity shadow-sm",
                        onclick: move |_| send_message(),
                        Send { size: 20, class: "ml-0.5" }
                    }
                }
            }
        }
    }
}
