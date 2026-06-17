use dioxus::prelude::*;
use dioxus_sdk_time::sleep;
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;

use crate::components::common::MarkdownContent;
use crate::components::ui::skeleton::Skeleton;
use crate::providers::current_user_id;

use super::pending_meal::PendingMealCard;
use super::state::{
    ChatActivity, ChatActivityKind, ChatContext, ChatMessage, ChatMessageAction, ChatMessageAttachment,
    SessionHistoryLoadPhase,
};
use api::conversation;

const MESSAGE_VIEWPORT_STYLE: &str = r#"#chat-message-viewport-wrapper {
  position: relative;
  overflow-y: auto;
}"#;

#[derive(Debug, Deserialize)]
struct ScrollProbe {
    near_top: bool,
}

#[component]
pub(super) fn ChatMessageList(has_pending_transition: bool) -> Element {
    let mut chat_state = use_context::<ChatContext>();
    let mut loading_more = use_signal(|| false);
    let mut near_top_tick = use_signal(|| 0_u64);
    let mut last_initial_scrolled_session = use_signal(String::new);
    let mut top_paging_enabled = use_signal(|| false);
    let active_session_id = chat_state
        .active_session_id
        .read()
        .clone()
        .unwrap_or_default();
    let messages = chat_state.messages.read().clone();
    let has_messages = !messages.is_empty();
    let messages_for_render = messages.clone();
    let active_session_for_initial_effect = active_session_id.clone();
    let active_session_for_reset_effect = active_session_id.clone();
    let active_session_for_listener_effect = active_session_id.clone();
    let should_auto_scroll_bottom = messages
        .last()
        .map(|message| message.is_streaming || message.is_skeleton)
        .unwrap_or(false);
    let scroll_signature = {
        let last = messages.last();
        format!(
            "{}:{}:{}:{}",
            active_session_id,
            last.map(|msg| msg.id).unwrap_or_default(),
            last.map(|msg| msg.text.len()).unwrap_or_default(),
            last.map(|msg| msg.is_streaming || msg.is_skeleton)
                .unwrap_or(false)
        )
    };

    use_effect(use_reactive((&active_session_for_reset_effect,), move |_| {
        top_paging_enabled.set(false);
        last_initial_scrolled_session.set(String::new());
        near_top_tick.set(0);
    }));

    use_effect(use_reactive(
        (
            &active_session_for_initial_effect,
            &scroll_signature,
            &has_messages,
            &should_auto_scroll_bottom,
        ),
        move |(session_for_initial, _signature, has_messages, should_auto_follow)| {
            let should_initial_scroll =
                has_messages && last_initial_scrolled_session() != session_for_initial;
            if !should_auto_follow && !should_initial_scroll {
                return;
            }
            if should_initial_scroll {
                top_paging_enabled.set(false);
                last_initial_scrolled_session.set(session_for_initial.clone());
            }
            document::eval(
                r#"
                const stickBottom = () => {
                    const viewport = document.querySelector('#chat-message-viewport-wrapper');
                    if (!viewport) return;
                    viewport.scrollTop = viewport.scrollHeight;
                };
                requestAnimationFrame(() => {
                    stickBottom();
                    requestAnimationFrame(stickBottom);
                    setTimeout(stickBottom, 80);
                    setTimeout(stickBottom, 180);
                    setTimeout(stickBottom, 320);
                });
                "#,
            );
            if should_initial_scroll {
                spawn(async move {
                    sleep(Duration::from_millis(420)).await;
                    top_paging_enabled.set(true);
                });
            }
        },
    ));

    use_effect(use_reactive((&active_session_for_listener_effect,), move |_| {
        let mut eval = document::eval(
            r#"
            let viewport = document.querySelector('#chat-message-viewport-wrapper');
            while (!viewport) {
                await new Promise((resolve) => requestAnimationFrame(resolve));
                viewport = document.querySelector('#chat-message-viewport-wrapper');
            }

            if (window.__warmmyChatViewport && window.__warmmyChatOnScroll) {
                window.__warmmyChatViewport.removeEventListener('scroll', window.__warmmyChatOnScroll);
            }

            const onScroll = () => {
                if (viewport.scrollTop <= 24) {
                    dioxus.send({ near_top: true });
                }
            };

            window.__warmmyChatViewport = viewport;
            window.__warmmyChatOnScroll = onScroll;
            viewport.addEventListener('scroll', onScroll, { passive: true });
            onScroll();
            await new Promise(() => {});
            "#,
        );

        spawn(async move {
            while let Ok(event) = eval.recv::<ScrollProbe>().await {
                if event.near_top {
                    near_top_tick.set(near_top_tick().saturating_add(1));
                }
            }
        });
    }));

    use_effect(use_reactive((&near_top_tick,), move |_| {
        if near_top_tick() == 0 || loading_more() || !top_paging_enabled() {
            return;
        }
        let Some(session_id) = chat_state.active_session_id.read().clone() else {
            return;
        };
        let history_window = chat_state
            .session_history_windows
            .read()
            .get(&session_id)
            .cloned()
            .unwrap_or_default();
        if history_window.phase == SessionHistoryLoadPhase::LoadingOlder {
            return;
        }
        if !history_window.has_more {
            return;
        }
        let Some(before_message_id) = history_window.next_before_message_id else {
            return;
        };
        {
            let mut windows = chat_state.session_history_windows.write();
            if let Some(window) = windows.get_mut(&session_id) {
                window.phase = SessionHistoryLoadPhase::LoadingOlder;
            }
        }
        let user_id = current_user_id();
        loading_more.set(true);
        spawn(async move {
            let page = conversation::get_session_history_cursor(
                user_id,
                session_id.clone(),
                conversation::SessionHistoryCursorInput {
                    limit: Some(8),
                    before_message_id: Some(before_message_id),
                },
            )
            .await;

            match page {
                Ok(page) => {
                    let existing = chat_state
                        .session_messages
                        .peek()
                        .get(&session_id)
                        .cloned()
                        .unwrap_or_default();
                    let existing_ids = existing
                        .iter()
                        .map(|message| message.id)
                        .collect::<HashSet<_>>();
                    let mut older = page
                        .items
                        .into_iter()
                        .enumerate()
                        .filter_map(|(offset, item)| {
                            let fallback = existing
                                .first()
                                .map(|message| message.id.saturating_sub(offset as u64 + 1))
                                .unwrap_or(offset as u64 + 1);
                            let mapped = ChatMessage {
                                id: item.id.parse::<u64>().unwrap_or(fallback),
                                text: item.content,
                                is_bot: item.role != "user",
                                is_skeleton: false,
                                is_streaming: false,
                                attachments: item
                                    .attachments
                                    .into_iter()
                                    .map(|attachment| ChatMessageAttachment {
                                        id: attachment.id,
                                        kind: attachment.kind,
                                        mime_type: attachment.mime_type,
                                        size_bytes: attachment.size_bytes,
                                        width: attachment.width,
                                        height: attachment.height,
                                        data_url: attachment.data_url,
                                        status: attachment.status,
                                    })
                                    .collect(),
                                action: None,
                                pending_meal: None,
                            };
                            if existing_ids.contains(&mapped.id) {
                                return None;
                            }
                            Some(mapped)
                        })
                        .collect::<Vec<_>>();

                    if !older.is_empty() {
                        older.extend(existing.clone());
                        chat_state
                            .session_messages
                            .write()
                            .insert(session_id.clone(), older.clone());
                        if chat_state
                            .active_session_id
                            .peek()
                            .as_ref()
                            .map(|active| active == &session_id)
                            .unwrap_or(false)
                        {
                            chat_state.messages.set(older);
                        }
                    }

                    let mut windows = chat_state.session_history_windows.write();
                    let window = windows.entry(session_id.clone()).or_default();
                    if window.end_index > window.start_index || window.has_more {
                        window.start_index = window.start_index.min(page.start_index);
                        window.end_index = window.end_index.max(page.end_index);
                    } else {
                        window.start_index = page.start_index;
                        window.end_index = page.end_index;
                    }
                    window.total_count = page.total_count;
                    window.next_before_message_id = page.next_before_message_id;
                    window.has_more = page.has_more;
                    window.phase = SessionHistoryLoadPhase::Ready;
                    drop(windows);

                    let mut probe_eval = document::eval(
                        r#"
                        const viewport = document.querySelector('#chat-message-viewport-wrapper');
                        dioxus.send({ near_top: viewport ? viewport.scrollTop <= 24 : false });
                        "#,
                    );
                    if let Ok(probe) = probe_eval.recv::<ScrollProbe>().await {
                        if probe.near_top {
                            near_top_tick.set(near_top_tick().saturating_add(1));
                        }
                    }
                }
                Err(_) => {
                    let mut windows = chat_state.session_history_windows.write();
                    if let Some(window) = windows.get_mut(&session_id) {
                        if window.phase == SessionHistoryLoadPhase::LoadingOlder {
                            window.phase = SessionHistoryLoadPhase::Ready;
                        }
                    }
                }
            }
            loading_more.set(false);
        });
    }));

    let activity = current_activity(chat_state);

    rsx! {
        div {
            id: "chat-message-viewport-wrapper",
            class: "flex-1 min-h-0 overflow-y-auto px-4 py-4 md:px-5 md:py-5",
            style { "{MESSAGE_VIEWPORT_STYLE}" }
            if !has_messages && has_pending_transition {
                PendingChatPlaceholder {}
            } else {
                div { class: "flex flex-col",
                    for msg in messages_for_render {
                        div { key: "{msg.id}", class: "pb-4",
                            ChatMessageBubble {
                                message: msg.clone(),
                                activity: if msg.is_bot && (msg.is_streaming || msg.is_skeleton) {
                                    activity.clone()
                                } else {
                                    None
                                },
                            }
                        }
                    }
                }
            }
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