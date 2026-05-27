use chrono::{Duration as ChronoDuration, NaiveDate};
use dioxus::prelude::*;
use dioxus_icons::lucide::{CalendarDays, Check, Send, Sparkles};
use dioxus_sdk_time::sleep;
use std::collections::HashSet;
use std::time::Duration;

use crate::blocks::current_user_id;
use crate::components::common::MarkdownContent;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::input::Input;
use crate::components::ui::skeleton::Skeleton;
use crate::today_session_id;
use api::conversation;
use api::meal;
use serde_json::Value;

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub text: String,
    pub is_bot: bool,
    pub is_skeleton: bool,
    pub is_streaming: bool,
    pub pending_meal: Option<meal::PendingMealLogDTO>,
}

pub static CHAT_MESSAGES: GlobalSignal<Vec<ChatMessage>> = Signal::global(Vec::new);
pub static ACTIVE_SESSION_ID: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CHAT_INPUT: GlobalSignal<String> = Signal::global(String::new);
pub static CHAT_NEXT_ID: GlobalSignal<u64> = Signal::global(|| 1_u64);

#[derive(Clone, PartialEq, Debug)]
pub struct PendingConversationMessage {
    pub session_id: String,
    pub content: String,
    pub started: bool,
}

#[derive(Clone, Copy)]
pub struct ConversationTransitionContext {
    pub pending: Signal<Option<PendingConversationMessage>>,
}

#[component]
pub fn ChatBlock(session_id: Option<String>) -> Element {
    let transition = try_consume_context::<ConversationTransitionContext>();
    let user_id = current_user_id();
    let current_session_id = session_id.clone().unwrap_or_else(today_session_id);
    let send_session_id = current_session_id.clone();
    let header_session_id = current_session_id.clone();
    let should_route_after_stream = session_id.is_none();
    let has_pending_transition = transition
        .map(|ctx| (ctx.pending)().is_some())
        .unwrap_or(false);

    let execute_user_id = user_id.clone();
    let execute_send = std::rc::Rc::new(move |content: String| {
        let sid = send_session_id.clone();
        let request_user_id = execute_user_id.clone();
        let route_after_stream = should_route_after_stream;
        let transition = transition;
        let active_sid = ACTIVE_SESSION_ID.read().clone();
        if active_sid.as_ref() != Some(&sid) {
            *ACTIVE_SESSION_ID.write() = Some(sid.clone());
            CHAT_MESSAGES.write().clear();
            *CHAT_NEXT_ID.write() = 1;
        }

        let bot_id = append_outgoing_message_pair(content.clone());
        let content_for_server = content;

        *ACTIVE_SESSION_ID.write() = Some(sid.clone());

        spawn(async move {
            match conversation::echo_stream(
                request_user_id.clone(),
                content_for_server.clone(),
                sid.clone(),
            )
            .await
            {
                Ok(stream) => {
                    append_agent_stream(stream, bot_id, sid.clone()).await;
                    if route_after_stream && is_active_session(&sid) {
                        navigator().replace(format!("/{}", sid));
                    }
                    if let Some(mut transition) = transition {
                        transition.pending.set(None);
                    }
                }
                Err(stream_err) => {
                    let fallback =
                        match conversation::echo(request_user_id, content_for_server, sid.clone())
                            .await
                        {
                            Ok(resp) => resp.reply,
                            Err(err) => {
                                format!("Server error: {stream_err}; fallback failed: {err}")
                            }
                        };
                    let mut all = CHAT_MESSAGES.write();
                    if is_active_session(&sid) {
                        if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                            bot_msg.is_skeleton = false;
                            bot_msg.is_streaming = false;
                            bot_msg.text = fallback;
                        }
                    }
                    drop(all);
                    if route_after_stream && is_active_session(&sid) {
                        navigator().replace(format!("/{}", sid));
                    }
                    if let Some(mut transition) = transition {
                        transition.pending.set(None);
                    }
                }
            }
        });
    });

    let history_session_id = current_session_id.clone();
    let history_user_id = user_id.clone();
    let history_is_detail_route = session_id.is_some();
    let execute_send_after_history = execute_send.clone();
    let transition_for_history = transition;
    let _history_loader = use_resource(use_reactive(
        (
            &history_user_id,
            &history_session_id,
            &history_is_detail_route,
        ),
        move |(request_user_id, sid, is_detail_route)| {
            let execute_send_after_history = execute_send_after_history.clone();
            let transition = transition_for_history;
            async move {
                if sid.is_empty() {
                    return;
                }

                *ACTIVE_SESSION_ID.write() = Some(sid.clone());
                let pending_message = transition
                    .and_then(|ctx| ctx.pending.peek().clone())
                    .filter(|pending| pending.session_id == sid);
                let has_pending = pending_message.is_some();

                let pending_meals = meal::list_pending_meals(request_user_id.clone(), sid.clone())
                    .await
                    .unwrap_or_default();

                match conversation::get_session_history(request_user_id.clone(), sid.clone()).await
                {
                    Ok(history) => {
                        if history.is_empty() {
                            if is_detail_route && !has_pending {
                                *CHAT_MESSAGES.write() = vec![ChatMessage {
                                    id: 0,
                                    text: "嗨！我是 warmmy，你的对话饮食助理。今天有什么想记录的，或者关于饮食健康的疑问吗？🍎".to_string(),
                                    is_bot: true,
                                    is_skeleton: false,
                                    is_streaming: false,
                                    pending_meal: None,
                                }];
                            } else {
                                append_pending_meal_messages(pending_meals, 1);
                            }
                            *CHAT_NEXT_ID.write() = next_chat_id();
                        } else {
                            if !is_detail_route && !has_pending {
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
                                    pending_meal: None,
                                });
                                current_next_id += 1;
                            }
                            for pending_meal in pending_meals {
                                loaded_msgs.push(ChatMessage {
                                    id: current_next_id,
                                    text: String::new(),
                                    is_bot: true,
                                    is_skeleton: false,
                                    is_streaming: false,
                                    pending_meal: Some(pending_meal),
                                });
                                current_next_id += 1;
                            }
                            *CHAT_MESSAGES.write() = loaded_msgs;
                            *CHAT_NEXT_ID.write() = current_next_id;
                        }

                        if let Some(pending) = pending_message {
                            if !pending.started {
                                if let Some(mut transition) = transition {
                                    transition.pending.with_mut(|current| {
                                        if let Some(current) = current {
                                            if current.session_id == pending.session_id
                                                && current.content == pending.content
                                            {
                                                current.started = true;
                                            }
                                        }
                                    });
                                }
                                execute_send_after_history(pending.content);
                            }
                        }
                    }
                    Err(_) => {
                        if !has_pending {
                            *CHAT_MESSAGES.write() = vec![ChatMessage {
                                id: 0,
                                text: "嗨！今天想吃点什么呢？".to_string(),
                                is_bot: true,
                                is_skeleton: false,
                                is_streaming: false,
                                pending_meal: None,
                            }];
                            append_pending_meal_messages(pending_meals, 1);
                            *CHAT_NEXT_ID.write() = next_chat_id();
                        } else {
                            append_pending_meal_messages(pending_meals, 1);
                            *CHAT_NEXT_ID.write() = next_chat_id();
                            if let Some(pending) = pending_message {
                                if !pending.started {
                                    if let Some(mut transition) = transition {
                                        transition.pending.with_mut(|current| {
                                            if let Some(current) = current {
                                                if current.session_id == pending.session_id
                                                    && current.content == pending.content
                                                {
                                                    current.started = true;
                                                }
                                            }
                                        });
                                    }
                                    execute_send_after_history(pending.content);
                                }
                            }
                        }
                    }
                }
            }
        },
    ));

    let execute_send_input = execute_send.clone();
    let send_message = std::rc::Rc::new(move || {
        if CHAT_MESSAGES
            .read()
            .iter()
            .any(|msg| msg.is_streaming || msg.is_skeleton)
        {
            return;
        }
        let content = CHAT_INPUT().trim().to_string();
        if content.is_empty() {
            return;
        }
        *CHAT_INPUT.write() = String::new();
        execute_send_input(content);
    });

    let send_message_keydown = send_message.clone();
    let send_message_click = send_message.clone();
    let is_streaming = use_memo(move || {
        CHAT_MESSAGES
            .read()
            .iter()
            .any(|msg| msg.is_streaming || msg.is_skeleton)
    });
    let mut finalizing_day = use_signal(|| false);
    let finalize_user_id = user_id.clone();
    let finalize_session_id = current_session_id.clone();
    let finalize_today = move |_| {
        if finalizing_day() || is_streaming() {
            return;
        }

        let request_user_id = finalize_user_id.clone();
        let request_session_id = finalize_session_id.clone();
        spawn(async move {
            finalizing_day.set(true);
            let bot_id = append_streaming_bot_slot();
            match meal::finalize_and_summarize_meal_day(request_user_id, request_session_id.clone())
                .await
            {
                Ok(stream) => {
                    append_agent_stream(stream, bot_id, request_session_id).await;
                }
                Err(err) => {
                    append_bot_text(format!("生成今日总结失败：{err}"));
                }
            }
            finalizing_day.set(false);
        });
    };

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
                        div { class: "flex items-center gap-2",
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                class: "rounded-full border border-border bg-background px-3 py-2 text-xs font-semibold text-muted-foreground hover:text-foreground",
                                disabled: is_streaming() || finalizing_day(),
                                onclick: finalize_today,
                                Check { size: 14 }
                                if finalizing_day() { "总结中" } else { "敲定今日" }
                            }
                            span {
                                class: "hidden rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-muted-foreground md:inline-flex",
                                "{header_session_id}"
                            }
                        }
                    }
                    if session_id.is_some() {
                        SessionStrip {
                            user_id: user_id.clone(),
                            active_session_id: header_session_id.clone(),
                        }
                    }
                }

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
                                disabled: is_streaming(),
                                oninput: move |e: FormEvent| *CHAT_INPUT.write() = e.value(),
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter && !is_streaming() {
                                        send_message_keydown();
                                    }
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Icon,
                                class: "rounded-full bg-foreground p-3 text-background shadow-sm hover:opacity-90",
                                disabled: is_streaming(),
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
fn PendingChatPlaceholder() -> Element {
    rsx! {
        div {
            class: "rounded-[1.5rem] border border-dashed border-border bg-background p-4 text-sm leading-relaxed text-muted-foreground",
            "正在加载今天的上下文，然后会把你的新消息接在历史记录之后。"
        }
    }
}

fn append_pending_meal_messages(pending_meals: Vec<meal::PendingMealLogDTO>, start_id: u64) {
    if pending_meals.is_empty() {
        return;
    }

    let mut messages = CHAT_MESSAGES.write();
    let mut next_id = start_id.max(next_chat_id());
    for pending_meal in pending_meals {
        if messages.iter().any(|message| {
            message.pending_meal.as_ref().map(|meal| &meal.id) == Some(&pending_meal.id)
        }) {
            continue;
        }
        messages.push(ChatMessage {
            id: next_id,
            text: String::new(),
            is_bot: true,
            is_skeleton: false,
            is_streaming: false,
            pending_meal: Some(pending_meal),
        });
        next_id += 1;
    }
    *CHAT_NEXT_ID.write() = next_id;
}

fn next_chat_id() -> u64 {
    CHAT_MESSAGES
        .read()
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn append_outgoing_message_pair(content: String) -> u64 {
    let mut messages = CHAT_MESSAGES.write();
    let user_id = CHAT_NEXT_ID();
    let bot_id = user_id.saturating_add(1);
    *CHAT_NEXT_ID.write() = bot_id.saturating_add(1);
    messages.push(ChatMessage {
        id: user_id,
        text: content,
        is_bot: false,
        is_skeleton: false,
        is_streaming: false,
        pending_meal: None,
    });
    messages.push(ChatMessage {
        id: bot_id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        pending_meal: None,
    });
    bot_id
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
fn SessionStrip(user_id: String, active_session_id: String) -> Element {
    let sessions = use_resource(move || {
        let request_user_id = user_id.clone();
        async move {
            api::conversation::list_user_sessions(request_user_id)
                .await
                .unwrap_or_default()
        }
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

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum ChatStreamWireEvent {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "interaction_requested")]
    InteractionRequested { interaction: AgentInteractionDTO },
}

#[derive(Clone, Debug, serde::Deserialize)]
struct AgentInteractionDTO {
    id: String,
    kind: String,
    payload: Value,
}

enum ChatStreamEvent {
    TextDelta(String),
    InteractionRequested(AgentInteractionDTO),
}

#[derive(Default)]
struct ChatStreamParser {
    buffer: String,
}

impl ChatStreamParser {
    fn parse(&mut self, chunk: &str) -> Vec<ChatStreamEvent> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();

        while let Some(index) = self.buffer.find('\n') {
            let line = self.buffer[..index].trim().to_string();
            self.buffer.drain(..=index);
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<ChatStreamWireEvent>(&line) {
                Ok(ChatStreamWireEvent::TextDelta { text }) => {
                    events.push(ChatStreamEvent::TextDelta(text));
                }
                Ok(ChatStreamWireEvent::InteractionRequested { interaction }) => {
                    events.push(ChatStreamEvent::InteractionRequested(interaction));
                }
                Err(_) => events.push(ChatStreamEvent::TextDelta(line)),
            }
        }

        events
    }
}

fn is_active_session(session_id: &str) -> bool {
    ACTIVE_SESSION_ID
        .read()
        .as_ref()
        .map(|active| active == session_id)
        .unwrap_or(false)
}

fn handle_interaction_requested(interaction: AgentInteractionDTO) {
    if interaction.kind == "meal_log_confirmation" {
        match serde_json::from_value::<meal::PendingMealLogDTO>(interaction.payload) {
            Ok(meal) => push_pending_meal_message(meal),
            Err(err) => append_bot_text(format!("无法渲染待确认操作：{err}")),
        }
    } else {
        append_bot_text(format!(
            "收到暂不支持的操作请求：{} ({})",
            interaction.kind, interaction.id
        ));
    }
}

fn push_pending_meal_message(pending_meal: meal::PendingMealLogDTO) {
    let mut messages = CHAT_MESSAGES.write();
    if messages
        .iter()
        .any(|message| message.pending_meal.as_ref().map(|meal| &meal.id) == Some(&pending_meal.id))
    {
        return;
    }
    let id = CHAT_NEXT_ID();
    *CHAT_NEXT_ID.write() = id.saturating_add(1);
    messages.push(ChatMessage {
        id,
        text: String::new(),
        is_bot: true,
        is_skeleton: false,
        is_streaming: false,
        pending_meal: Some(pending_meal),
    });
}

fn append_bot_text(text: String) {
    let id = CHAT_NEXT_ID();
    *CHAT_NEXT_ID.write() = id.saturating_add(1);
    CHAT_MESSAGES.write().push(ChatMessage {
        id,
        text,
        is_bot: true,
        is_skeleton: false,
        is_streaming: false,
        pending_meal: None,
    });
}

fn append_streaming_bot_slot() -> u64 {
    let id = CHAT_NEXT_ID();
    *CHAT_NEXT_ID.write() = id.saturating_add(1);
    CHAT_MESSAGES.write().push(ChatMessage {
        id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        pending_meal: None,
    });
    id
}

async fn append_agent_stream(
    mut stream: dioxus::fullstack::payloads::TextStream,
    bot_id: u64,
    session_id: String,
) {
    let mut first = true;
    let mut parser = ChatStreamParser::default();
    while let Some(chunk) = stream.next().await {
        if !is_active_session(&session_id) {
            return;
        }
        match chunk {
            Ok(text) => {
                if text.is_empty() {
                    continue;
                }
                for event in parser.parse(&text) {
                    match event {
                        ChatStreamEvent::TextDelta(delta) => {
                            let mut all = CHAT_MESSAGES.write();
                            if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                                if first {
                                    bot_msg.is_skeleton = false;
                                    first = false;
                                }
                                bot_msg.text.push_str(&delta);
                            }
                        }
                        ChatStreamEvent::InteractionRequested(interaction) => {
                            handle_interaction_requested(interaction);
                        }
                    }
                }
            }
            Err(err) => {
                if !is_active_session(&session_id) {
                    return;
                }
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

    if !is_active_session(&session_id) {
        return;
    }
    let mut all = CHAT_MESSAGES.write();
    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
        bot_msg.is_skeleton = false;
        bot_msg.is_streaming = false;
    }
}

#[component]
fn PendingMealCard(pending_meal: meal::PendingMealLogDTO) -> Element {
    let user_id = current_user_id();
    let session_id = ACTIVE_SESSION_ID().unwrap_or_else(today_session_id);
    let confirm_session_id = session_id.clone();
    let reject_session_id = session_id.clone();
    let mut saving = use_signal(|| false);
    let mut rejected = use_signal(|| pending_meal.status == "rejected");
    let mut confirmed = use_signal(|| pending_meal.status == "confirmed");
    let day_cycle = use_signal(|| pending_meal.day_cycle.clone());
    let mut foods = use_signal(|| pending_meal.foods.clone());
    let mut nutrition = use_signal(|| pending_meal.nutrition.clone());
    let mut previewing = use_signal(|| false);

    let update_preview = {
        let user_id = user_id.clone();
        let pending_id = pending_meal.id.clone();
        move || {
            let request_user_id = user_id.clone();
            let input = meal::ConfirmPendingMealInput {
                pending_id: pending_id.clone(),
                day_cycle: day_cycle(),
                foods: foods(),
            };
            spawn(async move {
                previewing.set(true);
                match meal::preview_pending_meal(request_user_id, input).await {
                    Ok(updated) => {
                        nutrition.set(updated.nutrition);
                    }
                    Err(err) => append_bot_text(format!("更新营养估算失败：{err}")),
                }
                previewing.set(false);
            });
        }
    };

    let confirm_meal = {
        let pending_id = pending_meal.id.clone();
        let user_id = user_id.clone();
        move |_| {
            let request_user_id = user_id.clone();
            let request_session_id = confirm_session_id.clone();
            let input = meal::ConfirmPendingMealInput {
                pending_id: pending_id.clone(),
                day_cycle: day_cycle(),
                foods: foods(),
            };
            spawn(async move {
                saving.set(true);
                let bot_id = append_streaming_bot_slot();
                match meal::confirm_pending_meal(request_user_id, request_session_id.clone(), input)
                    .await
                {
                    Ok(stream) => {
                        confirmed.set(true);
                        append_agent_stream(stream, bot_id, request_session_id).await;
                    }
                    Err(err) => append_bot_text(format!("确认用餐记录失败：{err}")),
                }
                saving.set(false);
            });
        }
    };

    let reject_meal = {
        let pending_id = pending_meal.id.clone();
        let user_id = user_id.clone();
        move |_| {
            let request_user_id = user_id.clone();
            let request_session_id = reject_session_id.clone();
            let request_pending_id = pending_id.clone();
            spawn(async move {
                saving.set(true);
                let bot_id = append_streaming_bot_slot();
                match meal::reject_pending_meal(
                    request_user_id,
                    request_session_id.clone(),
                    request_pending_id,
                )
                .await
                {
                    Ok(stream) => {
                        rejected.set(true);
                        append_agent_stream(stream, bot_id, request_session_id).await;
                    }
                    Err(err) => append_bot_text(format!("取消用餐记录失败：{err}")),
                }
                saving.set(false);
            });
        }
    };

    rsx! {
        div { class: "rounded-[1.5rem] border border-border bg-background p-4 text-sm shadow-none",
            div { class: "flex items-start justify-between gap-3",
                div {
                    div { class: "text-base font-semibold text-foreground", "请确认这次用餐记录" }
                    p { class: "mt-1 text-xs leading-relaxed text-muted-foreground", "agent 只创建了待确认记录，确认后才会写入 meal log。" }
                }
                span { class: "rounded-full border border-border px-3 py-1 text-xs text-muted-foreground", "{day_cycle}" }
            }
            div { class: "mt-4 space-y-2",
                for (index, food) in foods().into_iter().enumerate() {
                    div { key: "{pending_meal.id}:{index}", class: "grid grid-cols-[1fr_80px_80px] gap-2",
                        Input {
                            class: "rounded-xl border border-border bg-card px-3 py-2 text-sm",
                            value: food.name.clone(),
                            oninput: {
                                let update_preview = update_preview.clone();
                                move |e: FormEvent| {
                                    foods.with_mut(|items| {
                                        if let Some(item) = items.get_mut(index) {
                                            item.name = e.value();
                                        }
                                    });
                                    update_preview();
                                }
                            },
                        }
                        Input {
                            class: "rounded-xl border border-border bg-card px-3 py-2 text-sm",
                            value: food.quantity.to_string(),
                            oninput: {
                                let update_preview = update_preview.clone();
                                move |e: FormEvent| {
                                    foods.with_mut(|items| {
                                        if let Some(item) = items.get_mut(index) {
                                            item.quantity = e.value().parse::<f32>().unwrap_or(item.quantity);
                                        }
                                    });
                                    update_preview();
                                }
                            },
                        }
                        Input {
                            class: "rounded-xl border border-border bg-card px-3 py-2 text-sm",
                            value: food.unit.clone(),
                            oninput: {
                                let update_preview = update_preview.clone();
                                move |e: FormEvent| {
                                    foods.with_mut(|items| {
                                        if let Some(item) = items.get_mut(index) {
                                            item.unit = e.value();
                                        }
                                    });
                                    update_preview();
                                }
                            },
                        }
                    }
                }
            }
            div { class: "mt-4 rounded-xl border border-border bg-card px-3 py-2 text-xs leading-relaxed text-muted-foreground",
                if previewing() {
                    "正在更新估算..."
                } else {
                    "估算：{nutrition().calories:.0} kcal · 蛋白质 {nutrition().protein_g:.1}g · 碳水 {nutrition().carbs_g:.1}g · 脂肪 {nutrition().fat_g:.1}g"
                }
            }
            div { class: "mt-4 flex flex-wrap gap-2",
                Button {
                    class: "rounded-xl bg-foreground text-background",
                    disabled: saving() || confirmed() || rejected(),
                    onclick: confirm_meal,
                    if confirmed() { "已确认" } else { "确认记录" }
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    class: "rounded-xl border border-border",
                    disabled: saving() || confirmed() || rejected(),
                    onclick: reject_meal,
                    if rejected() { "已取消" } else { "取消" }
                }
            }
        }
    }
}
