mod ai_model;

use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, Bot, Sparkles};

use crate::blocks::CurrentUserContext;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};

use ai_model::AIModelBlock;

#[component]
pub fn WarmmyBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let current_user_id = (current_user.user_id)();
    let nav = navigator();

    rsx! {
        div { class: "h-full min-h-0 overflow-y-auto bg-background px-4 py-5 pb-28 text-foreground md:px-8 md:py-8 md:pb-12",
            div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6",
                section { class: "relative overflow-hidden rounded-[1rem] border border-border bg-card px-5 py-6 md:px-8 md:py-8",
                    div { class: "absolute -right-16 -top-20 h-52 w-52 rounded-full bg-[#ffad1a]/20 blur-3xl" }
                    div { class: "absolute -bottom-24 left-8 h-56 w-56 rounded-full bg-[#0f7a4d]/10 blur-3xl" }
                    div { class: "relative flex flex-col gap-6 md:flex-row md:items-start md:justify-between",
                        div { class: "max-w-3xl",
                            div { class: "mb-4 inline-flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted-foreground",
                                Sparkles { size: 14 }
                                "Warmmy agent"
                            }
                            h1 { class: "font-doodle text-4xl font-semibold leading-none tracking-[-0.9px] text-foreground md:text-6xl md:tracking-[-1.5px]",
                                "配置 Warmmy 的模型能力"
                            }
                            p { class: "mt-4 max-w-2xl text-base leading-relaxed text-muted-foreground md:text-lg",
                                "这里集中维护 Warmmy agent 使用的对话、RAG 嵌入和图像识别模型。页面不出现在主导航，只从个人页进入。"
                            }
                        }
                        div { class: "flex shrink-0 gap-2",
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                class: "rounded-full border border-border px-4 text-foreground hover:bg-muted/50",
                                onclick: move |_| {
                                    nav.push("/me");
                                },
                                ArrowLeft { size: 16 }
                                "返回 Me"
                            }
                            div { class: "hidden h-10 w-10 items-center justify-center rounded-full bg-foreground text-background shadow-sm sm:flex",
                                Bot { size: 18 }
                            }
                        }
                    }
                }

                div { key: "{current_user_id}", class: "grid grid-cols-1 gap-6",
                    AIModelBlock { user_id: current_user_id }
                }
            }
        }
    }
}
