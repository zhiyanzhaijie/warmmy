use chrono::{Duration as ChronoDuration, NaiveDate};
use dioxus::prelude::*;
use dioxus_icons::lucide::{CalendarDays, Send, Sparkles};
use dioxus_sdk_time::sleep;
use std::collections::HashSet;
use std::time::Duration;

use crate::components::common::MarkdownContent;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::input::Input;
use crate::components::ui::skeleton::Skeleton;
use crate::today_session_id;
use crate::views::FIRST_MSG;
use api::conversation;

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub text: String,
    pub is_bot: bool,
    pub is_skeleton: bool,
    pub is_streaming: bool,
}

pub static CHAT_MESSAGES: GlobalSignal<Vec<ChatMessage>> = Signal::global(Vec::new);
pub static ACTIVE_SESSION_ID: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CHAT_INPUT: GlobalSignal<String> = Signal::global(String::new);
pub static CHAT_NEXT_ID: GlobalSignal<u64> = Signal::global(|| 1_u64);

#[component]
pub fn ChatBlock(session_id: Option<String>) -> Element {
    let session_id_send = session_id.clone();
    let current_session_id = session_id.clone().unwrap_or_else(today_session_id);
    let send_session_id = current_session_id.clone();
    let header_session_id = current_session_id.clone();

    let execute_send = std::rc::Rc::new(move |content: String| {
        let sid = send_session_id.clone();
        let active_sid = ACTIVE_SESSION_ID.read().clone();
        if active_sid.as_ref() != Some(&sid) {
            *ACTIVE_SESSION_ID.write() = Some(sid.clone());
            CHAT_MESSAGES.write().clear();
            *CHAT_NEXT_ID.write() = 1;
        }

        let user_id = CHAT_NEXT_ID();
        *CHAT_NEXT_ID.write() = user_id + 1;
        CHAT_MESSAGES.write().push(ChatMessage {
            id: user_id,
            text: content.clone(),
            is_bot: false,
            is_skeleton: false,
            is_streaming: false,
        });

        let bot_id = CHAT_NEXT_ID();
        *CHAT_NEXT_ID.write() = bot_id + 1;
        CHAT_MESSAGES.write().push(ChatMessage {
            id: bot_id,
            text: String::new(),
            is_bot: true,
            is_skeleton: true,
            is_streaming: true,
        });

        let content_for_server = content;

        *ACTIVE_SESSION_ID.write() = Some(sid.clone());

        if session_id_send.is_none() {
            navigator().replace(format!("/{}", sid));
        }

        spawn(async move {
            match conversation::echo_stream(content_for_server.clone(), sid.clone()).await {
                Ok(mut stream) => {
                    let mut first = true;
                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(text) => {
                                if text.is_empty() {
                                    continue;
                                }
                                let mut all = CHAT_MESSAGES.write();
                                if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                                    if first {
                                        bot_msg.is_skeleton = false;
                                        first = false;
                                    }
                                    bot_msg.text.push_str(&text);
                                }
                            }
                            Err(err) => {
                                let mut all = CHAT_MESSAGES.write();
                                if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                                    bot_msg.is_skeleton = false;
                                    bot_msg.is_streaming = false;
                                    bot_msg.text.push_str(&format!("\n[stream error] {err}"));
                                }
                                return;
                            }
                        }
                    }
                    let mut all = CHAT_MESSAGES.write();
                    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                        bot_msg.is_skeleton = false;
                        bot_msg.is_streaming = false;
                    }
                }
                Err(stream_err) => {
                    let fallback = match conversation::echo(content_for_server, sid).await {
                        Ok(resp) => resp.reply,
                        Err(err) => format!("Server error: {stream_err}; fallback failed: {err}"),
                    };
                    let mut all = CHAT_MESSAGES.write();
                    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                        bot_msg.is_skeleton = false;
                        bot_msg.is_streaming = false;
                        bot_msg.text = fallback;
                    }
                }
            }
        });
    });

    let history_session_id = current_session_id.clone();
    let history_is_detail_route = session_id.is_some();
    let execute_send_after_history = execute_send.clone();
    let _history_loader = use_resource(use_reactive(
        (&history_session_id, &history_is_detail_route),
        move |(sid, is_detail_route)| {
            let execute_send_after_history = execute_send_after_history.clone();
            async move {
                if sid.is_empty() {
                    return;
                }

                *ACTIVE_SESSION_ID.write() = Some(sid.clone());
                let pending_first_msg = FIRST_MSG.peek().clone();

                match conversation::get_session_history(sid.clone()).await {
                    Ok(history) => {
                        if history.is_empty() {
                            if is_detail_route && pending_first_msg.is_none() {
                                *CHAT_MESSAGES.write() = vec![ChatMessage {
                                    id: 0,
                                    text: "嗨！我是 warmmy，你的对话饮食助理。今天有什么想记录的，或者关于饮食健康的疑问吗？🍎".to_string(),
                                    is_bot: true,
                                    is_skeleton: false,
                                    is_streaming: false,
                                }];
                            } else {
                                *CHAT_MESSAGES.write() = Vec::new();
                            }
                            *CHAT_NEXT_ID.write() = 1;
                        } else {
                            if !is_detail_route {
                                navigator().replace(format!("/{}", sid));
                            }
                            let mut loaded_msgs = Vec::new();
                            let mut current_next_id = 1_u64;
                            for msg in history {
                                loaded_msgs.push(ChatMessage {
                                    id: current_next_id,
                                    text: msg.content,
                                    is_bot: msg.role != "user",
                                    is_skeleton: false,
                                    is_streaming: false,
                                });
                                current_next_id += 1;
                            }
                            *CHAT_MESSAGES.write() = loaded_msgs;
                            *CHAT_NEXT_ID.write() = current_next_id;
                        }

                        if let Some(content) = pending_first_msg {
                            *FIRST_MSG.write() = None;
                            execute_send_after_history(content);
                        }
                    }
                    Err(_) => {
                        if pending_first_msg.is_none() {
                            *CHAT_MESSAGES.write() = vec![ChatMessage {
                                id: 0,
                                text: "嗨！今天想吃点什么呢？".to_string(),
                                is_bot: true,
                                is_skeleton: false,
                                is_streaming: false,
                            }];
                            *CHAT_NEXT_ID.write() = 1;
                        } else if let Some(content) = pending_first_msg {
                            *FIRST_MSG.write() = None;
                            execute_send_after_history(content);
                        }
                    }
                }
            }
        },
    ));

    let execute_send_input = execute_send.clone();
    let send_message = std::rc::Rc::new(move || {
        let content = CHAT_INPUT().trim().to_string();
        if content.is_empty() {
            return;
        }
        *CHAT_INPUT.write() = String::new();
        execute_send_input(content);
    });

    let send_message_keydown = send_message.clone();
    let send_message_click = send_message.clone();

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
            class: "h-full min-h-0 overflow-hidden px-0 md:px-8 md:py-6",
            Card {
                class: "mx-auto flex h-full min-h-0 max-w-4xl flex-col overflow-hidden rounded-none border-x border-border bg-card/80 shadow-none md:rounded-[2rem] md:border",

                div {
                    class: "border-b border-border bg-card/95 backdrop-blur",
                    div {
                        class: "flex items-start justify-between gap-4 px-5 py-4 md:px-6",
                        div {
                            p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Conversation memory" }
                            h2 {
                                class: "mt-1 flex items-center gap-2 font-doodle text-2xl font-semibold tracking-[-0.6px] text-foreground",
                                Sparkles { size: 22, class: "text-[#b86b10]" }
                                "Diet Buddy"
                            }
                        }
                        span {
                            class: "hidden rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-muted-foreground md:inline-flex",
                            "{header_session_id}"
                        }
                    }
                    if session_id.is_some() {
                        SessionStrip { active_session_id: header_session_id.clone() }
                    }
                }

                div {
                    id: "chat-message-viewport",
                    class: "flex-1 min-h-0 overflow-y-auto space-y-5 px-4 py-5 md:px-6 md:py-6",
                    for msg in CHAT_MESSAGES().iter() {
                        ChatMessageBubble {
                            key: "{msg.id}",
                            message: msg.clone(),
                        }
                    }
                    div { id: "chat-message-bottom", class: "h-px w-full" }
                }

                div {
                    class: "border-t border-border bg-card/95 px-4 pb-5 pt-3 backdrop-blur md:px-6 md:pb-6",
                    div {
                        class: "rounded-[1.75rem] border border-border bg-background p-2 focus-within:shadow-md",
                        div {
                            class: "flex items-center gap-2",
                            Input {
                                class: "flex-1 border-none bg-transparent px-4 py-3 font-medium text-foreground shadow-none outline-none placeholder:text-muted-foreground",
                                r#type: "text",
                                placeholder: "记录餐食，或询问下一顿吃什么...",
                                value: CHAT_INPUT(),
                                oninput: move |e: FormEvent| *CHAT_INPUT.write() = e.value(),
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        send_message_keydown();
                                    }
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Icon,
                                class: "rounded-full bg-foreground p-3 text-background shadow-sm hover:opacity-90",
                                onclick: move |_| send_message_click(),
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
fn ChatMessageBubble(message: ChatMessage) -> Element {
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
                "{message.text}"
            }
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
    } else {
        if is_streaming {
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
}

#[component]
fn SessionStrip(active_session_id: String) -> Element {
    let sessions = use_resource(move || async move {
        api::conversation::list_user_sessions()
            .await
            .unwrap_or_default()
    });

    let session_list = sessions.read().clone().unwrap_or_default();
    let session_days: HashSet<String> = session_list.into_iter().collect();
    let days = recent_session_days(7)
        .into_iter()
        .map(|day| {
            let has_session = session_days.contains(&day);
            (day, has_session)
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            class: "overflow-x-auto px-4 pb-3 hide-scrollbar",
            div {
                class: "flex min-w-max gap-2",
                for (day, has_session) in days {
                    SessionChip {
                        session_id: day,
                        active_session_id: active_session_id.clone(),
                        has_session,
                    }
                }
            }
        }
    }
}

#[component]
fn SessionChip(session_id: String, active_session_id: String, has_session: bool) -> Element {
    let is_active = session_id == active_session_id;
    let label = session_label(&session_id);

    if is_active {
        rsx! {
            button {
                r#type: "button",
                disabled: true,
                class: "inline-flex items-center gap-2 rounded-full bg-foreground px-3 py-2 text-xs font-semibold text-background shadow-sm opacity-95",
                CalendarDays { size: 14 }
                "{label}"
            }
        }
    } else if has_session {
        let sid = session_id.clone();
        rsx! {
            button {
                r#type: "button",
                onclick: move |_| { navigator().push(format!("/{sid}")); },
                class: "inline-flex items-center gap-2 rounded-full border border-border bg-background px-3 py-2 text-xs font-semibold text-muted-foreground transition-colors hover:border-foreground/30 hover:text-foreground",
                CalendarDays { size: 14 }
                "{label}"
            }
        }
    } else {
        rsx! {
            button {
                r#type: "button",
                disabled: true,
                class: "inline-flex items-center gap-2 rounded-full border border-dashed border-border bg-transparent px-3 py-2 text-xs font-semibold text-muted-foreground opacity-45",
                CalendarDays { size: 14 }
                "{label}"
            }
        }
    }
}

fn recent_session_days(count: i64) -> Vec<String> {
    let today = today_session_id();
    let Some(today) = NaiveDate::parse_from_str(&today, "%Y-%m-%d").ok() else {
        return vec![today_session_id()];
    };

    (0..count)
        .map(|offset| {
            (today - ChronoDuration::days(offset))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect()
}

fn session_label(session_id: &str) -> String {
    if session_id == today_session_id() {
        "今天".to_string()
    } else {
        session_id
            .rsplit_once('-')
            .map(|(prefix, day)| {
                prefix
                    .rsplit_once('-')
                    .map(|(_, month)| format!("{month}/{day}"))
                    .unwrap_or_else(|| session_id.to_string())
            })
            .unwrap_or_else(|| session_id.to_string())
    }
}
