use crate::blocks::{ACTIVE_SESSION_ID, CHAT_INPUT, CHAT_MESSAGES, CHAT_NEXT_ID};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::input::Input;
use crate::today_session_id;
use crate::views::FIRST_MSG;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Cloud, Send, Sparkles};

#[component]
pub fn HomeView() -> Element {
    let mut input = use_signal(String::new);

    let mut start_chat_with_msg = move || {
        let content = input().trim().to_string();
        if content.is_empty() {
            return;
        }

        input.set(String::new());
        let session_id = today_session_id();

        // Clear stale detail state before route transition; ChatBlock will render the optimistic send.
        *ACTIVE_SESSION_ID.write() = Some(session_id.clone());
        CHAT_MESSAGES.write().clear();
        CHAT_INPUT.write().clear();
        *CHAT_NEXT_ID.write() = 1;

        // Set FIRST_MSG to let ChatBlock initiate the stream.
        *FIRST_MSG.write() = Some(content);

        // Transition to detail URL of today's date; ChatBlock consumes FIRST_MSG there.
        navigator().push(format!("/{session_id}"));
    };

    rsx! {
        div {
            class: "h-full min-h-0 overflow-y-auto px-5 py-8 pb-28 md:px-10 md:py-12 md:pb-12",
            div {
                class: "mx-auto flex min-h-full w-full max-w-6xl flex-col justify-center gap-8",
                div {
                    class: "grid grid-cols-1 items-center gap-8 md:grid-cols-2 md:gap-12",
                    div {
                        class: "flex flex-col items-start text-left",
                        p {
                            class: "mb-4 rounded-full border border-border bg-card px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground",
                            "Warmmy local-first food agent"
                        }
                        h1 {
                            class: "font-doodle text-5xl font-semibold leading-none tracking-[-1.2px] text-foreground md:text-6xl lg:text-7xl",
                            "Eat with memory,"
                            br {}
                            "not guesses."
                        }
                        p {
                            class: "mt-6 max-w-xl text-base leading-relaxed text-muted-foreground md:text-lg",
                            "记录你的餐食、偏好、忌口和健康期望。Warmmy 会把这些长期事实带进每一次饮食建议。"
                        }

                        div {
                            class: "mt-8 w-full max-w-xl",
                            div {
                                class: "rounded-[2rem] border border-border bg-card p-2 shadow-none",
                                div {
                                    class: "flex items-center gap-2 rounded-[1.5rem] border border-border bg-background p-2 focus-within:shadow-md",
                                    Input {
                                        class: "flex-1 border-none bg-transparent px-4 py-3 text-base font-medium text-foreground shadow-none outline-none placeholder:text-muted-foreground",
                                        r#type: "text",
                                        placeholder: "我中午吃了绿豆排骨汤300g...",
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
                                        class: "rounded-full bg-foreground p-3 text-background shadow-sm hover:opacity-90",
                                        onclick: move |_| start_chat_with_msg(),
                                        Send { size: 20, class: "ml-0.5" }
                                    }
                                }
                            }
                        }

                        div {
                            class: "mt-6 flex flex-wrap gap-2 text-xs text-muted-foreground",
                            span { class: "rounded-full border border-border bg-card px-3 py-1", "偏好记忆" }
                            span { class: "rounded-full border border-border bg-card px-3 py-1", "餐食记录" }
                            span { class: "rounded-full border border-border bg-card px-3 py-1", "健康期望" }
                        }
                    }

                    div {
                        class: "order-first flex justify-center md:order-last",
                        Card {
                            class: "relative aspect-square w-full max-w-[360px] overflow-hidden rounded-[3rem] border border-border bg-card shadow-none md:max-w-[420px]",
                            div {
                                class: "absolute inset-6 rounded-[2.4rem] border border-border bg-background",
                                style: "background: radial-gradient(circle at 22% 20%, rgba(255, 173, 26, 0.22), transparent 12rem), radial-gradient(circle at 80% 85%, rgba(15, 122, 77, 0.13), transparent 11rem), var(--background);",
                            }
                            div {
                                class: "absolute left-8 top-8 rounded-full border border-border bg-card px-4 py-2 text-xs font-semibold text-muted-foreground",
                                "today context"
                            }
                            div {
                                class: "absolute bottom-8 left-8 right-8 rounded-[1.75rem] border border-border bg-card p-5",
                                Sparkles { class: "mb-4 text-[#b86b10] w-10 h-10" }
                                h2 { class: "font-doodle text-3xl font-semibold tracking-[-0.8px] text-foreground", "下一餐建议" }
                                p { class: "mt-2 text-sm leading-relaxed text-muted-foreground", "基于你刚记录的餐食、忌口和当前健康期望生成。" }
                            }
                            div {
                                class: "absolute right-10 top-14 text-[#b86b10] opacity-50",
                                Cloud { class: "h-12 w-24" }
                            }
                        }
                    }
                }
            }
        }
    }
}
