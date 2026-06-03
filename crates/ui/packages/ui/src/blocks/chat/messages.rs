use dioxus::prelude::*;
use dioxus_sdk_time::sleep;
use std::time::Duration;

use crate::components::common::MarkdownContent;
use crate::components::ui::skeleton::Skeleton;

use super::pending_meal::PendingMealCard;
use super::state::{
    ChatActivity, ChatActivityKind, ChatContext, ChatMessage, ChatMessageAction,
    ChatMessageAttachment,
};

#[component]
pub(super) fn ChatMessageList(has_pending_transition: bool) -> Element {
    let chat_state = use_context::<ChatContext>();
    let scroll_signature = use_memo(move || {
        let messages = chat_state.messages.read();
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

    let activity = current_activity(chat_state);

    rsx! {
        div {
            id: "chat-message-viewport",
            class: "flex-1 min-h-0 overflow-y-auto space-y-4 px-4 py-4 md:px-5 md:py-5",
            if chat_state.messages.read().is_empty() && has_pending_transition {
                PendingChatPlaceholder {}
            } else {
                for msg in chat_state.messages.read().iter() {
                    ChatMessageBubble {
                        key: "{msg.id}",
                        message: msg.clone(),
                        activity: if msg.is_bot && (msg.is_streaming || msg.is_skeleton) {
                            activity.clone()
                        } else {
                            None
                        },
                    }
                }
            }
            div { id: "chat-message-bottom", class: "h-px w-full" }
        }
    }
}

fn current_activity(chat_state: ChatContext) -> Option<ChatActivity> {
    let session_id = chat_state.active_session_id.read().clone()?;
    chat_state
        .session_activities
        .read()
        .get(&session_id)
        .cloned()
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
fn ChatMessageBubble(message: ChatMessage, activity: Option<ChatActivity>) -> Element {
    if let Some(pending_meal) = message.pending_meal {
        return rsx! {
            div { class: "max-w-[92%] md:max-w-[78%]",
                PendingMealCard { pending_meal }
            }
        };
    }

    if message.is_bot {
        let action = message.action.clone();
        rsx! {
            div {
                class: "max-w-[92%] rounded-[1.5rem] rounded-tl-sm bg-card/80 p-4 text-[15px] font-medium leading-relaxed text-foreground shadow-none md:max-w-[76%]",
                ChatActivityBlock { activity }
                StreamMessage {
                    text: message.text,
                    is_skeleton: message.is_skeleton,
                    is_streaming: message.is_streaming,
                }
                if let Some(action) = action {
                    ChatMessageActionButton { action }
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "ml-auto max-w-[88%] rounded-[1.5rem] rounded-tr-sm bg-foreground p-4 text-[0.80rem] font-medium leading-relaxed text-background shadow-sm whitespace-pre-wrap md:max-w-[70%]",
                MessageAttachmentGrid { attachments: message.attachments }
                if !message.text.trim().is_empty() {
                    div { class: "mt-3 first:mt-0", "{message.text}" }
                }
            }
        }
    }
}

#[component]
fn ChatActivityBlock(activity: Option<ChatActivity>) -> Element {
    let is_visible = activity.is_some();
    let tone = activity
        .as_ref()
        .map(|activity| match activity.kind {
            ChatActivityKind::ReadingInput => "bg-sky-500",
            ChatActivityKind::UsingTool => "bg-amber-500",
            ChatActivityKind::SavingMemory => "bg-emerald-500",
            ChatActivityKind::Persisting => "bg-zinc-500",
            ChatActivityKind::WaitingUser => "bg-violet-500",
            ChatActivityKind::Cancelling => "bg-rose-500",
            ChatActivityKind::Thinking | ChatActivityKind::CallingModel => "bg-foreground",
        })
        .unwrap_or("bg-foreground");
    let class = if is_visible {
        "mb-3 max-h-16 border-l-2 border-border bg-background/55 px-3 py-2 opacity-100"
    } else {
        "mb-0 max-h-0 border-l-2 border-transparent bg-background/0 px-3 py-0 opacity-0"
    };

    rsx! {
        div {
            class: "overflow-hidden text-xs font-medium leading-relaxed text-muted-foreground transition-[max-height,opacity,margin,padding,border-color,background-color] duration-300 ease-out {class}",
            if let Some(activity) = activity {
                div { class: "flex min-w-0 items-center gap-2",
                    span { class: "relative flex h-2 w-2 shrink-0",
                        span { class: "absolute inline-flex h-full w-full animate-ping rounded-full opacity-50 {tone}" }
                        span { class: "relative inline-flex h-2 w-2 rounded-full {tone}" }
                    }
                    span { class: "min-w-0 truncate", "{activity.label}" }
                }
            }
        }
    }
}

#[component]
fn ChatMessageActionButton(action: ChatMessageAction) -> Element {
    let nav = navigator();
    let route = action.route.clone();

    rsx! {
        button {
            r#type: "button",
            class: "mt-3 inline-flex items-center rounded-full border border-border bg-background px-3.5 py-2 text-xs font-semibold text-foreground shadow-sm transition-colors hover:border-foreground/30 hover:bg-muted active:scale-[0.98]",
            onclick: move |_| {
                nav.push(route.clone());
            },
            "{action.label}"
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
