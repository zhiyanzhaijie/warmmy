use super::warmmy_pisa_svg::WarmmyPisaSvg;
use crate::blocks::ChatActionContext;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::textarea::{Textarea, TextareaVariant};
use crate::today_session_id;
use chrono::{Datelike, Local};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Send, Sparkles};

#[component]
pub fn HomeView() -> Element {
    let mut input = use_signal(String::new);
    let chat_actions = use_context::<ChatActionContext>();
    let nav = navigator();
    let today_index = Local::now().weekday().num_days_from_monday() as usize;

    let start_chat_with_msg = move || {
        let content = input().trim().to_string();
        if content.is_empty() {
            return;
        }

        input.set(String::new());
        let session_id = today_session_id();
        chat_actions
            .send_message
            .call(session_id.clone(), content, Vec::new(), false);
        nav.push(format!("/{session_id}"));
    };
    let mut start_chat_keydown = start_chat_with_msg.clone();
    let mut start_chat_click = start_chat_with_msg.clone();

    rsx! {
        div { class: "relative h-full min-h-0 overflow-hidden",
            div {
                class: "pointer-events-none absolute inset-0",
                style: "background:
                    radial-gradient(circle at 18% 14%, rgba(255, 173, 26, 0.16), transparent 22rem),
                    radial-gradient(circle at 82% 18%, rgba(15, 122, 77, 0.10), transparent 20rem),
                    linear-gradient(180deg, transparent 0%, color-mix(in oklab, var(--background) 86%, transparent) 62%, var(--background) 100%);",
            }
            PizzaWeekBackground { active_index: today_index }
            div { class: "pointer-events-none absolute inset-0 z-0 h-full w-full opacity-60 dark:opacity-40",
                img {
                    src: asset!("/assets/human-warmmy.svg"),
                    class: "absolute left-1/2 top-[4%] w-[130%] -translate-x-1/2 object-contain md:bottom-[-10%] md:w-[100%] lg:top-[-18%] lg:w-[82%] 2xl:w-[72%]",
                    alt: "",
                }
            }
            div { class: "relative flex h-full min-h-0 flex-col justify-end px-4 pb-5 md:px-10 md:pb-10",
                div { class: "mx-auto flex w-full max-w-3xl flex-col gap-5",
                    div { class: "max-w-2xl",
                        div { class: "mb-4 flex h-12 w-12 items-center justify-center rounded-[1.25rem] border border-border bg-card/75 text-foreground backdrop-blur md:h-14 md:w-14",
                            Sparkles { size: 24 }
                        }
                        p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "你好呀" }
                        h1 { class: "mt-3 font-doodle text-4xl font-semibold leading-none text-foreground md:text-6xl",
                            "今天你吃饭了吗"
                        }
                        p { class: "mt-4 max-w-xl text-sm leading-relaxed text-muted-foreground md:text-base",
                            "我们只有补充好能量，才能移山平河哦"
                        }
                    }

                    div { class: "w-full rounded-[2rem] border border-border bg-card/80 p-2 shadow-lg backdrop-blur",
                        div { class: "flex items-end gap-2 rounded-[1.5rem] border border-border bg-background/95 p-2 focus-within:shadow-md",
                            Textarea {
                                variant: TextareaVariant::Ghost,
                                class: "max-h-40 min-h-12 min-w-0 flex-1 resize-none overflow-y-auto border-none bg-transparent px-4 py-3 text-base font-medium leading-relaxed text-foreground shadow-none outline-none placeholder:text-muted-foreground placeholder:whitespace-nowrap placeholder:overflow-hidden placeholder:text-ellipsis [field-sizing:content]",
                                rows: "1",
                                placeholder: "和屋米说一下吃的，想吃的吧......",
                                value: input(),
                                oninput: move |e: FormEvent| {
                                    input.set(e.value());
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter && !e.modifiers().shift() {
                                        start_chat_keydown();
                                    }
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Icon,
                                class: "mb-1 rounded-full bg-foreground p-3 text-background shadow-sm hover:opacity-90",
                                onclick: move |_| start_chat_click(),
                                Send { size: 20, class: "ml-0.5" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PizzaWeekBackground(active_index: usize) -> Element {
    rsx! {
        div { class: "pointer-events-none absolute left-1/2 top-[8%] h-[min(82vw,560px)] w-[min(82vw,560px)] -translate-x-1/2 opacity-80 md:top-[6%]",
            WarmmyPisaSvg { active_index }
        }
    }
}
