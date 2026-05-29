use dioxus::prelude::*;
use dioxus_sdk_time::sleep;
use std::time::Duration;

use crate::components::common::MarkdownContent;
use crate::components::ui::skeleton::Skeleton;

use super::pending_meal::PendingMealCard;
use super::state::{ChatMessage, ChatMessageAttachment, CHAT_MESSAGES};

#[component]
pub(super) fn ChatMessageList(has_pending_transition: bool) -> Element {
    let scroll_signature = use_memo(move || {
        let messages = CHAT_MESSAGES.read();
        let last = messages.last();
        format!(
            "{}:{}:{}",
            messages.len(),
            last.map(|msg| msg.text.len()).unwrap_or_default(),
            last.map(|msg| msg.is_streaming || msg.is_skeleton)
                .unwrap_or_default()
        )
    });

    use_effect(move || {
        let _ = scroll_signature();
        document::eval(
            r#"
            requestAnimationFrame(() => {
                const viewport = document.getElementById("chat-message-viewport");
                const anchor = document.getElementById("chat-message-bottom");
                if (!viewport || !anchor) return;
                anchor.scrollIntoView({ block: "end", behavior: "smooth" });
            });
            "#,
        );
    });

    rsx! {
        div {
            id: "chat-message-viewport",
            class: "flex-1 min-h-0 overflow-y-auto space-y-5 px-4 py-5 md:px-6 md:py-6",
            if CHAT_MESSAGES().is_empty() && has_pending_transition {
                PendingChatPlaceholder {}
            } else {
                for msg in CHAT_MESSAGES().iter() {
                    ChatMessageBubble {
                        key: "{msg.id}",
                        message: msg.clone(),
                    }
                }
            }
            div { id: "chat-message-bottom", class: "h-px w-full" }
        }
    }
}

#[component]
fn PendingChatPlaceholder() -> Element {
    rsx! {
        div {
            class: "rounded-[1.5rem] border border-dashed border-border bg-background p-4 text-sm leading-relaxed text-muted-foreground",
            "正在加载今天的上下文，然后会把你的新消息接在历史记录之后。"
        }
    }
}

#[component]
fn ChatMessageBubble(message: ChatMessage) -> Element {
    if let Some(pending_meal) = message.pending_meal {
        return rsx! {
            div { class: "max-w-[92%] md:max-w-[78%]",
                PendingMealCard { pending_meal }
            }
        };
    }

    if message.is_bot {
        rsx! {
            div {
                class: "max-w-[90%] rounded-[1.5rem] rounded-tl-sm border border-border bg-background p-4 text-[15px] font-medium leading-relaxed text-foreground shadow-none md:max-w-[76%]",
                StreamMessage {
                    text: message.text,
                    is_skeleton: message.is_skeleton,
                    is_streaming: message.is_streaming,
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "ml-auto max-w-[86%] rounded-[1.5rem] rounded-tr-sm bg-foreground p-4 text-[15px] font-medium leading-relaxed text-background shadow-sm whitespace-pre-wrap md:max-w-[70%]",
                MessageAttachmentGrid { attachments: message.attachments }
                if !message.text.trim().is_empty() {
                    div { class: "mt-3 first:mt-0", "{message.text}" }
                }
            }
        }
    }
}

#[component]
fn MessageAttachmentGrid(attachments: Vec<ChatMessageAttachment>) -> Element {
    let images = attachments
        .into_iter()
        .filter(|attachment| attachment.kind == "image")
        .collect::<Vec<_>>();

    rsx! {
        if !images.is_empty() {
            div { class: "grid max-w-[17rem] grid-cols-2 gap-2",
                for image in images {
                    if image.status == "available" {
                        if let Some(data_url) = image.data_url.clone() {
                            img {
                                key: "{image.id}",
                                class: "aspect-square w-full rounded-lg border border-background/20 object-cover",
                                src: data_url,
                                alt: "用户上传的图片",
                            }
                        } else {
                            MissingImageTile { id: image.id }
                        }
                    } else {
                        MissingImageTile { id: image.id }
                    }
                }
            }
        }
    }
}

#[component]
fn MissingImageTile(id: String) -> Element {
    rsx! {
        div {
            key: "{id}",
            class: "flex aspect-square w-full items-center justify-center rounded-lg border border-background/20 bg-background/10 px-3 text-center text-xs leading-relaxed text-background/70",
            "图片已清理"
        }
    }
}

#[component]
fn StreamMessage(text: String, is_skeleton: bool, is_streaming: bool) -> Element {
    let initial_text = text.clone();
    let mut visible_text = use_signal(move || {
        if is_streaming {
            String::new()
        } else {
            initial_text
        }
    });

    use_effect(use_reactive(
        (&text, &is_streaming),
        move |(text, is_streaming)| {
            if !is_streaming {
                visible_text.set(text.clone());
                return;
            }

            if text.len() < visible_text.peek().len() {
                visible_text.set(String::new());
            }

            spawn(async move {
                loop {
                    let current_len = visible_text.peek().len();
                    if current_len >= text.len() {
                        return;
                    }

                    let mut next_len = text.len();
                    let mut chars_seen = 0;
                    for (idx, _) in text[current_len..].char_indices() {
                        chars_seen += 1;
                        if chars_seen == 4 {
                            next_len = current_len + idx;
                            break;
                        }
                    }

                    visible_text.set(text[..next_len].to_string());
                    sleep(Duration::from_millis(18)).await;
                }
            });
        },
    ));

    if is_skeleton && text.is_empty() {
        rsx! {
            div {
                class: "flex flex-col gap-2 w-48",
                Skeleton { class: "h-4 w-full rounded" }
                Skeleton { class: "h-4 w-[80%] rounded" }
            }
        }
    } else if is_streaming {
        rsx! {
            div { class: "relative",
                MarkdownContent { src: visible_text, class: "chat-stream-markdown".to_string() }
                span { class: "chat-stream-caret", " " }
            }
        }
    } else {
        rsx! {
            div { class: "relative",
                MarkdownContent { src: text, class: "chat-stream-markdown".to_string() }
            }
        }
    }
}
