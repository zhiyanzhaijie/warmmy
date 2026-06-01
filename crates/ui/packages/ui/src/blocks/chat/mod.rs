mod composer;
mod messages;
mod pending_meal;
mod sessions;
mod state;
mod stream;

use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, Check};
use std::rc::Rc;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::providers::current_user_id;
use crate::today_session_id;

use api::conversation;
use api::meal;

use composer::{ChatComposer, SendChatMessage};
use messages::ChatMessageList;
use sessions::SessionStrip;
use stream::{
    activate_session, append_bot_text, append_pending_meal_messages, append_streaming_bot_slot,
    set_active_session_messages,
};

pub use state::{
    ChatMessage, ChatRuntimeContext, ChatStateContext, ConversationTransitionContext,
    PendingConversationMessage, SendConversationMessage,
};

pub(crate) use state::ComposerImageAttachment;
pub(crate) use stream::{
    activate_session as activate_chat_session, append_agent_stream,
    append_bot_text as append_chat_bot_text, append_outgoing_message_pair,
};

#[component]
pub fn ChatBlock(session_id: Option<String>) -> Element {
    let transition = try_consume_context::<ConversationTransitionContext>();
    let chat_state = use_context::<ChatStateContext>();
    let user_id = current_user_id();
    let current_session_id = session_id.clone().unwrap_or_else(today_session_id);
    let send_session_id = current_session_id.clone();
    let header_session_id = current_session_id.clone();
    let should_route_after_stream = session_id.is_none();
    let runtime = use_context::<ChatRuntimeContext>();
    let has_pending_transition = transition
        .map(|ctx| (ctx.pending)().is_some())
        .unwrap_or(false);

    let execute_send = Rc::new(
        move |content: String, attachments: Vec<ComposerImageAttachment>| {
            let sid = send_session_id.clone();
            if let Some(sender) = runtime.send_message.read().clone() {
                sender.call(sid, content, attachments, should_route_after_stream);
            }
        },
    );

    start_pending_transition(
        current_session_id.clone(),
        chat_state,
        transition,
        execute_send.clone(),
    );

    load_session_history(
        user_id.clone(),
        current_session_id.clone(),
        session_id.is_some(),
        chat_state,
        transition,
        execute_send.clone(),
    );

    let is_streaming = use_memo(move || {
        chat_state
            .messages
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
        dioxus::core::spawn_forever(async move {
            finalizing_day.set(true);
            let bot_id = append_streaming_bot_slot(chat_state, request_session_id.clone());
            match meal::finalize_and_summarize_meal_day(request_user_id, request_session_id.clone())
                .await
            {
                Ok(stream) => {
                    append_agent_stream(chat_state, stream, bot_id, request_session_id).await;
                }
                Err(err) => {
                    append_bot_text(chat_state, request_session_id, format!("生成今日总结失败：{err}"));
                }
            }
            finalizing_day.set(false);
        });
    };

    rsx! {
        div {
            class: "h-full min-h-0 overflow-hidden bg-background md:px-4 md:py-4",
            div {
                class: "mx-auto flex h-full min-h-0 max-w-5xl flex-col overflow-hidden bg-background md:rounded-[1.5rem] md:bg-card/35",
                ChatHeader {
                    user_id: user_id.clone(),
                    session_id: session_id.clone(),
                    active_session_id: header_session_id,
                    is_streaming: is_streaming(),
                    finalizing_day: finalizing_day(),
                    on_finalize: finalize_today,
                }
                ChatMessageList { has_pending_transition }
                ChatComposer {
                    is_streaming: is_streaming(),
                    on_send: SendChatMessage::new(execute_send),
                }
            }
        }
    }
}

#[component]
fn ChatHeader(
    user_id: String,
    session_id: Option<String>,
    active_session_id: String,
    is_streaming: bool,
    finalizing_day: bool,
    on_finalize: EventHandler<MouseEvent>,
) -> Element {
    let nav = navigator();
    rsx! {
        div {
            class: "border-b border-border bg-card/70 backdrop-blur md:bg-card/45",
            div {
                class: "flex items-center justify-between gap-3 px-4 pb-1.5 pt-2.5 md:px-5 md:pt-3",
                div { class: "flex min-w-0 items-center gap-2",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        class: "rounded-full text-muted-foreground hover:bg-muted hover:text-foreground",
                        onclick: move |_| {
                            nav.push("/");
                        },
                        ArrowLeft { size: 18 }
                    }
                    WarmmyMascotIcon { active: is_streaming || finalizing_day }
                    div { class: "min-w-0 pl-1",
                        p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Conversation memory" }
                    }
                }
                div { class: "flex items-center gap-2",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "rounded-full bg-foreground px-3 py-2 text-xs font-semibold text-background shadow-sm hover:opacity-90",
                        disabled: is_streaming || finalizing_day,
                        onclick: move |event| on_finalize.call(event),
                        Check { size: 14 }
                        if finalizing_day { "总结中" } else { "敲定今日" }
                    }
                    span {
                        class: "hidden rounded-full bg-muted px-3 py-1 text-xs font-medium text-muted-foreground md:inline-flex",
                        "{active_session_id}"
                    }
                }
            }
            if session_id.is_some() {
                SessionStrip {
                    user_id,
                    active_session_id,
                }
            }
        }
    }
}

#[component]
fn WarmmyMascotIcon(active: bool) -> Element {
    rsx! {
        svg {
            class: "h-12 w-12 shrink-0 text-foreground md:h-14 md:w-14",
            view_box: "0 0 64 64",
            role: "presentation",
            "aria-hidden": "true",
            path {
                d: "M 12 35 C 12 23, 22 15, 33 14 C 45 13, 53 22, 52 35 C 51 47, 42 53, 31 52 C 20 51, 12 46, 12 35 Z",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "3.2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
            path {
                d: "M 15 21 C 17 13, 21 7, 25 5 C 30 8, 33 12, 35 17 C 28 15, 21 17, 15 21 Z",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "3.4",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
            path {
                d: "M 48 20 C 47 13, 44 8, 40 6 C 35 9, 32 12, 30 17 C 37 15, 43 16, 48 20 Z",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "3.4",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
            g {
                if active {
                    animateTransform {
                        attribute_name: "transform",
                        r#type: "translate",
                        values: "-4 1; 2 -1; 5 1; -2 0; -4 1",
                        dur: "1.35s",
                        repeat_count: "indefinite",
                    }
                    animateTransform {
                        attribute_name: "transform",
                        r#type: "rotate",
                        values: "-7 32 33; 4 32 33; 8 32 33; -3 32 33; -7 32 33",
                        dur: "1.35s",
                        additive: "sum",
                        repeat_count: "indefinite",
                    }
                }
                line {
                    x1: "25",
                    y1: "30",
                    x2: "25",
                    y2: "42",
                    stroke: "currentColor",
                    stroke_width: "6",
                    stroke_linecap: "round",
                    if !active {
                        animate {
                            attribute_name: "y1",
                            values: "30;30;41;30;30",
                            key_times: "0;0.80;0.82;0.84;1",
                            dur: "3.4s",
                            repeat_count: "indefinite",
                        }
                    }
                }
                line {
                    x1: "41",
                    y1: "30",
                    x2: "41",
                    y2: "42",
                    stroke: "currentColor",
                    stroke_width: "6",
                    stroke_linecap: "round",
                    if !active {
                        animate {
                            attribute_name: "y1",
                            values: "30;30;41;30;30",
                            key_times: "0;0.80;0.82;0.84;1",
                            dur: "3.4s",
                            repeat_count: "indefinite",
                        }
                    }
                }
            }
        }
    }
}

fn load_session_history(
    history_user_id: String,
    history_session_id: String,
    history_is_detail_route: bool,
    chat_state: ChatStateContext,
    transition: Option<ConversationTransitionContext>,
    execute_send_after_history: Rc<dyn Fn(String, Vec<ComposerImageAttachment>)>,
) {
    let _history_loader = use_resource(use_reactive(
        (
            &history_user_id,
            &history_session_id,
            &history_is_detail_route,
        ),
        move |(request_user_id, sid, is_detail_route)| {
            let transition = transition;
            let execute_send_after_history = execute_send_after_history.clone();
            async move {
                if sid.is_empty() {
                    return;
                }

                activate_session(chat_state, sid.clone());
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
                                set_active_session_messages(
                                    chat_state,
                                    sid.clone(),
                                    vec![ChatMessage {
                                        id: 0,
                                        text: "嗨！我是 warmmy，你的对话饮食助理。今天有什么想记录的，或者关于饮食健康的疑问吗？🍎".to_string(),
                                        is_bot: true,
                                        is_skeleton: false,
                                        is_streaming: false,
                                        attachments: Vec::new(),
                                        pending_meal: None,
                                    }],
                                    1,
                                );
                            } else {
                                append_pending_meal_messages(chat_state, sid.clone(), pending_meals, 1);
                            }
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
                                    attachments: msg
                                        .attachments
                                        .into_iter()
                                        .map(|attachment| state::ChatMessageAttachment {
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
                                    attachments: Vec::new(),
                                    pending_meal: Some(pending_meal),
                                });
                                current_next_id += 1;
                            }
                            set_active_session_messages(
                                chat_state,
                                sid.clone(),
                                loaded_msgs,
                                current_next_id,
                            );
                        }

                        send_pending_transition(
                            pending_message,
                            transition,
                            execute_send_after_history,
                        );
                    }
                    Err(_) => {
                        if !has_pending {
                            set_active_session_messages(
                                chat_state,
                                sid.clone(),
                                vec![ChatMessage {
                                    id: 0,
                                    text: "嗨！今天想吃点什么呢？".to_string(),
                                    is_bot: true,
                                    is_skeleton: false,
                                    is_streaming: false,
                                    attachments: Vec::new(),
                                    pending_meal: None,
                                }],
                                1,
                            );
                            append_pending_meal_messages(chat_state, sid.clone(), pending_meals, 1);
                        } else {
                            append_pending_meal_messages(chat_state, sid.clone(), pending_meals, 1);
                            send_pending_transition(
                                pending_message,
                                transition,
                                execute_send_after_history,
                            );
                        }
                    }
                }
            }
        },
    ));
}

fn start_pending_transition(
    current_session_id: String,
    chat_state: ChatStateContext,
    transition: Option<ConversationTransitionContext>,
    execute_send_after_history: Rc<dyn Fn(String, Vec<ComposerImageAttachment>)>,
) {
    let pending_message = transition
        .and_then(|ctx| ctx.pending.peek().clone())
        .filter(|pending| pending.session_id == current_session_id);
    send_pending_transition(pending_message, transition, execute_send_after_history);
    let _ = chat_state;
}

fn send_pending_transition(
    pending_message: Option<PendingConversationMessage>,
    transition: Option<ConversationTransitionContext>,
    execute_send_after_history: Rc<dyn Fn(String, Vec<ComposerImageAttachment>)>,
) {
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
            execute_send_after_history(pending.content, Vec::new());
        }
    }
}
