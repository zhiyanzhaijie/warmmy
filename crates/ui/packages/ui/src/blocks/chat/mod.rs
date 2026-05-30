mod composer;
mod messages;
mod pending_meal;
mod sessions;
mod state;
mod stream;

use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, Check};
use std::rc::Rc;

use crate::blocks::current_user_id;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::today_session_id;

use api::conversation;
use api::meal;

use composer::{ChatComposer, SendChatMessage};
use messages::ChatMessageList;
use sessions::SessionStrip;
use stream::{
    append_agent_stream, append_bot_text, append_outgoing_message_pair,
    append_pending_meal_messages, append_streaming_bot_slot, is_active_session, next_chat_id,
};

pub use state::{
    ChatMessage, ConversationTransitionContext, PendingConversationMessage, ACTIVE_SESSION_ID,
    CHAT_INPUT, CHAT_MESSAGES, CHAT_NEXT_ID,
};

use state::ComposerImageAttachment;

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
    let execute_send = Rc::new(
        move |content: String, attachments: Vec<ComposerImageAttachment>| {
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

            let bot_id = append_outgoing_message_pair(content.clone(), attachments.clone());
            let content_for_server = content;
            let attachments_for_server = attachments;

            *ACTIVE_SESSION_ID.write() = Some(sid.clone());

            spawn(async move {
                let mut uploaded_attachments = Vec::new();
                for attachment in attachments_for_server {
                    let uploaded = conversation::store_ephemeral_image(
                        request_user_id.clone(),
                        sid.clone(),
                        attachment.mime_type.clone(),
                        attachment.bytes,
                        None,
                        None,
                    )
                    .await;
                    match uploaded {
                        Ok(image) => {
                            uploaded_attachments.push(conversation::ChatImageAttachmentInput {
                                asset_id: image.asset_id,
                                mime_type: image.mime_type,
                                size_bytes: image.size_bytes,
                                width: image.width,
                                height: image.height,
                                preview_data_url: Some(attachment.preview_data_url),
                            });
                        }
                        Err(err) => {
                            let mut all = CHAT_MESSAGES.write();
                            if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                                bot_msg.is_skeleton = false;
                                bot_msg.is_streaming = false;
                                bot_msg.text = format!("图片上传失败：{err}");
                            }
                            return;
                        }
                    }
                }

                let send_input = conversation::ChatSendInput {
                    text: content_for_server.clone(),
                    attachments: uploaded_attachments,
                };
                match conversation::echo_stream(
                    request_user_id.clone(),
                    send_input.clone(),
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
                        let fallback = match conversation::echo(
                            request_user_id,
                            send_input,
                            sid.clone(),
                        )
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
        },
    );

    load_session_history(
        user_id.clone(),
        current_session_id.clone(),
        session_id.is_some(),
        transition,
        execute_send.clone(),
    );

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
                    } else {
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
            let execute_send_after_history = execute_send_after_history.clone();
            let transition = transition;
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
                                    attachments: Vec::new(),
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
                            *CHAT_MESSAGES.write() = loaded_msgs;
                            *CHAT_NEXT_ID.write() = current_next_id;
                        }

                        send_pending_transition(
                            pending_message,
                            transition,
                            execute_send_after_history,
                        );
                    }
                    Err(_) => {
                        if !has_pending {
                            *CHAT_MESSAGES.write() = vec![ChatMessage {
                                id: 0,
                                text: "嗨！今天想吃点什么呢？".to_string(),
                                is_bot: true,
                                is_skeleton: false,
                                is_streaming: false,
                                attachments: Vec::new(),
                                pending_meal: None,
                            }];
                            append_pending_meal_messages(pending_meals, 1);
                            *CHAT_NEXT_ID.write() = next_chat_id();
                        } else {
                            append_pending_meal_messages(pending_meals, 1);
                            *CHAT_NEXT_ID.write() = next_chat_id();
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
