use dioxus::prelude::*;
use dioxus_sdk_time::sleep;
use futures_util::future::{select, Either};
use futures_util::StreamExt;
use std::rc::Rc;
use std::time::Duration;

use api::conversation;
use dioxus::prelude::ServerFnError;

use crate::blocks::{
    activate_chat_session, append_agent_stream, append_chat_bot_text,
    append_outgoing_message_pair, append_streaming_bot_slot, ChatActionContext, ChatContext,
    ChatMessageAction, ComposerImageAttachment, DEFAULT_STREAM_IDLE_TIMEOUT,
    FinalizeConversationDay, IMAGE_STREAM_IDLE_TIMEOUT, SendConversationMessage,
};

use super::current_user_id;
use api::meal;

const CHAT_STREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(45);
const CHAT_FALLBACK_TIMEOUT: Duration = Duration::from_secs(45);

#[component]
pub fn ChatRuntimeProvider(children: Element) -> Element {
    let chat_state = use_context::<ChatContext>();
    let user_id = current_user_id();

    let runtime = use_coroutine(move |mut rx: UnboundedReceiver<ChatRuntimeCommand>| async move {
        while let Some(command) = rx.next().await {
            match command {
                ChatRuntimeCommand::Send {
                    request_user_id,
                    session_id,
                    content,
                    attachments,
                    route_after_stream,
                } => {
                    send_conversation_message(
                        chat_state,
                        request_user_id,
                        session_id,
                        content,
                        attachments,
                        route_after_stream,
                    )
                    .await;
                }
                ChatRuntimeCommand::FinalizeDay {
                    request_user_id,
                    session_id,
                } => {
                    finalize_conversation_day(chat_state, request_user_id, session_id).await;
                }
            }
        }
    });
    let runtime_tx = runtime.tx();

    let send_message = use_hook(move || {
        let request_user_id = user_id.clone();
        let tx = runtime_tx.clone();
        SendConversationMessage::new(Rc::new(
            move |session_id: String,
                  content: String,
                  attachments: Vec<ComposerImageAttachment>,
                  route_after_stream: bool| {
                let _ = tx.unbounded_send(ChatRuntimeCommand::Send {
                    request_user_id: request_user_id.clone(),
                    session_id,
                    content,
                    attachments,
                    route_after_stream,
                });
            },
        ))
    });
    let finalize_tx = runtime.tx();
    let finalize_day = use_hook(move || {
        FinalizeConversationDay::new(Rc::new(move |request_user_id: String, session_id: String| {
            let _ = finalize_tx.unbounded_send(ChatRuntimeCommand::FinalizeDay {
                request_user_id,
                session_id,
            });
        }))
    });
    use_context_provider(|| ChatActionContext {
        send_message,
        finalize_day,
    });

    rsx! {
        {children}
    }
}

enum ChatRuntimeCommand {
    Send {
        request_user_id: String,
        session_id: String,
        content: String,
        attachments: Vec<ComposerImageAttachment>,
        route_after_stream: bool,
    },
    FinalizeDay {
        request_user_id: String,
        session_id: String,
    },
}

async fn finalize_conversation_day(
    mut chat_state: ChatContext,
    request_user_id: String,
    session_id: String,
) {
    if (chat_state.finalizing_day)() {
        return;
    }

    chat_state.finalizing_day.set(true);
    let bot_id = append_streaming_bot_slot(chat_state, session_id.clone());
    match meal::finalize_and_summarize_meal_day(request_user_id, session_id.clone()).await {
        Ok(stream) => {
            append_agent_stream(
                chat_state,
                stream,
                bot_id,
                session_id,
                DEFAULT_STREAM_IDLE_TIMEOUT,
            )
            .await;
        }
        Err(err) => {
            append_chat_bot_text(chat_state, session_id, format!("生成今日总结失败：{err}"));
        }
    }
    chat_state.finalizing_day.set(false);
}

async fn send_conversation_message(
    chat_state: ChatContext,
    request_user_id: String,
    session_id: String,
    content: String,
    attachments: Vec<ComposerImageAttachment>,
    route_after_stream: bool,
) {
    let session_has_streaming = chat_state
        .session_messages
        .peek()
        .get(&session_id)
        .map(|messages| {
            messages
                .iter()
                .any(|message| message.is_streaming || message.is_skeleton)
        })
        .unwrap_or(false);
    if session_has_streaming {
        return;
    }

    let active_sid = chat_state.active_session_id.read().clone();
    if active_sid.as_ref() != Some(&session_id) {
        activate_chat_session(chat_state, session_id.clone());
    }

    let bot_id = append_outgoing_message_pair(
        chat_state,
        session_id.clone(),
        content.clone(),
        attachments.clone(),
    );
    activate_chat_session(chat_state, session_id.clone());

    let mut uploaded_attachments = Vec::new();
    for attachment in attachments {
        let uploaded = conversation::store_ephemeral_image(
            request_user_id.clone(),
            session_id.clone(),
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
                stop_bot_with_text(
                    chat_state,
                    session_id.clone(),
                    bot_id,
                    friendly_chat_error(ChatErrorKind::from_server_error(&err)),
                );
                return;
            }
        }
    }

    let send_input = conversation::ChatSendInput {
        text: content,
        attachments: uploaded_attachments,
    };
    let idle_timeout = if send_input.attachments.is_empty() {
        DEFAULT_STREAM_IDLE_TIMEOUT
    } else {
        IMAGE_STREAM_IDLE_TIMEOUT
    };
    match echo_stream_with_timeout(
        request_user_id.clone(),
        send_input.clone(),
        session_id.clone(),
    )
    .await
    {
        Ok(stream) => {
            append_agent_stream(chat_state, stream, bot_id, session_id.clone(), idle_timeout).await;
            if route_after_stream {
                navigator().replace(format!("/{session_id}"));
            }
        }
        Err(stream_err) => {
            match echo_with_timeout(request_user_id, send_input, session_id.clone()).await {
                Ok(resp) => stop_bot_with_text(
                    chat_state,
                    session_id.clone(),
                    bot_id,
                    FriendlyChatError::text(resp.reply),
                ),
                Err(err) => stop_bot_with_text(
                    chat_state,
                    session_id.clone(),
                    bot_id,
                    friendly_chat_request_error(stream_err, err),
                ),
            }
            if route_after_stream {
                navigator().replace(format!("/{session_id}"));
            }
        }
    }
}

async fn echo_stream_with_timeout(
    user_id: String,
    input: conversation::ChatSendInput,
    session_id: String,
) -> Result<dioxus::fullstack::payloads::TextStream, ServerFnError> {
    let stream = conversation::echo_stream(user_id, input, session_id);
    let timeout = sleep(CHAT_STREAM_CONNECT_TIMEOUT);
    futures_util::pin_mut!(stream);
    futures_util::pin_mut!(timeout);

    match select(stream, timeout).await {
        Either::Left((result, _)) => result,
        Either::Right((_, _)) => Err(timeout_server_error(CHAT_STREAM_CONNECT_TIMEOUT)),
    }
}

async fn echo_with_timeout(
    user_id: String,
    input: conversation::ChatSendInput,
    session_id: String,
) -> Result<conversation::EchoResponse, ServerFnError> {
    let response = conversation::echo(user_id, input, session_id);
    let timeout = sleep(CHAT_FALLBACK_TIMEOUT);
    futures_util::pin_mut!(response);
    futures_util::pin_mut!(timeout);

    match select(response, timeout).await {
        Either::Left((result, _)) => result,
        Either::Right((_, _)) => Err(timeout_server_error(CHAT_FALLBACK_TIMEOUT)),
    }
}

fn timeout_server_error(duration: Duration) -> ServerFnError {
    ServerFnError::ServerError {
        message: format!("chat request timed out after {} seconds", duration.as_secs()),
        code: 408,
        details: Some(serde_json::json!({
            "code": "chat.timeout",
            "kind": "timeout",
        })),
    }
}

#[derive(Clone)]
struct FriendlyChatError {
    text: String,
    action: Option<ChatMessageAction>,
}

impl FriendlyChatError {
    fn text(text: String) -> Self {
        Self { text, action: None }
    }
}

fn friendly_chat_request_error(stream_err: ServerFnError, fallback_err: ServerFnError) -> FriendlyChatError {
    let stream_kind = ChatErrorKind::from_server_error(&stream_err);
    let fallback_kind = ChatErrorKind::from_server_error(&fallback_err);
    let kind = stream_kind.or(fallback_kind);
    if kind.is_none() {
        log_unclassified_chat_error("stream", &stream_err);
        log_unclassified_chat_error("fallback", &fallback_err);
    }
    friendly_chat_error_for_kind(kind, Some((&stream_err, &fallback_err)))
}

fn friendly_chat_error(kind: Option<ChatErrorKind>) -> FriendlyChatError {
    friendly_chat_error_for_kind(kind, None)
}

fn friendly_chat_error_for_kind(
    kind: Option<ChatErrorKind>,
    raw_errors: Option<(&ServerFnError, &ServerFnError)>,
) -> FriendlyChatError {
    match kind {
        Some(ChatErrorKind::ChatModelNotConfigured) => FriendlyChatError {
            text: "我还没有接好说话的小耳机。你帮我选一个聊天模型，我就能继续陪你聊。".to_string(),
            action: Some(ChatMessageAction {
                label: "去帮 warmmy 配好模型".to_string(),
                route: "/warmmy".to_string(),
            }),
        },
        Some(ChatErrorKind::VisionModelNotConfigured) => FriendlyChatError {
            text: "这张图片我想认真看看，但还没有配好看图用的小放大镜。你帮我接上视觉模型，我就能继续辨认啦。".to_string(),
            action: Some(ChatMessageAction {
                label: "去帮 warmmy 配好视觉模型".to_string(),
                route: "/warmmy".to_string(),
            }),
        },
        Some(ChatErrorKind::MemoryUnavailable) => FriendlyChatError {
            text: "我的记忆抽屉刚才有点卡住，没能好好读写这段对话。".to_string(),
            action: None,
        },
        Some(ChatErrorKind::ImageUnsupported) => FriendlyChatError {
            text: "这张图片我暂时看不清。换一张常见格式的图片，我会再认真辨认。".to_string(),
            action: None,
        },
        Some(ChatErrorKind::ModelUnavailable) => FriendlyChatError {
            text: "我暂时联系不上模型。可以检查一下 API key、base URL 和模型名称。".to_string(),
            action: Some(ChatMessageAction {
                label: "检查 warmmy 的模型设置".to_string(),
                route: "/warmmy".to_string(),
            }),
        },
        Some(ChatErrorKind::Timeout) => FriendlyChatError {
            text: "我刚才想了太久还没说出口。稍后再试，或者换一个响应更快的模型。".to_string(),
            action: Some(ChatMessageAction {
                label: "看看模型设置".to_string(),
                route: "/warmmy".to_string(),
            }),
        },
        Some(ChatErrorKind::EmptyInput) => FriendlyChatError {
            text: "我没听清你想说什么。重新告诉我一次就好。".to_string(),
            action: None,
        },
        None => {
            if let Some((stream_err, fallback_err)) = raw_errors {
                log_unclassified_chat_error("stream", stream_err);
                log_unclassified_chat_error("fallback", fallback_err);
            }
            FriendlyChatError {
            text: "我刚才有点走神。请稍后再试一次。".to_string(),
            action: None,
            }
        }
    }
}

fn log_unclassified_chat_error(stage: &str, error: &ServerFnError) {
    eprintln!("[warmmy chat] unclassified {stage} error: {error:?}");
}

#[derive(Clone, Copy)]
enum ChatErrorKind {
    ChatModelNotConfigured,
    VisionModelNotConfigured,
    MemoryUnavailable,
    ImageUnsupported,
    ModelUnavailable,
    Timeout,
    EmptyInput,
}

impl ChatErrorKind {
    fn from_server_error(error: &ServerFnError) -> Option<Self> {
        let ServerFnError::ServerError { details, .. } = error else {
            return None;
        };
        let code = details
            .as_ref()
            .and_then(|details| details.get("code"))
            .and_then(|code| code.as_str())?;
        match code {
            "ai.chat_not_configured" => Some(Self::ChatModelNotConfigured),
            "ai.vision_not_configured" => Some(Self::VisionModelNotConfigured),
            "ai.capability_not_configured" => Some(Self::ModelUnavailable),
            "memory.database_unavailable" => Some(Self::MemoryUnavailable),
            "media.image_unsupported" | "media.image_unavailable" => {
                Some(Self::ImageUnsupported)
            }
            "model.upstream_unavailable" => Some(Self::ModelUnavailable),
            "chat.timeout" => Some(Self::Timeout),
            "chat.empty_input" => Some(Self::EmptyInput),
            _ => None,
        }
    }
}

fn stop_bot_with_text(
    mut chat_state: ChatContext,
    session_id: String,
    bot_id: u64,
    error: FriendlyChatError,
) {
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
        bot_msg.is_skeleton = false;
        bot_msg.is_streaming = false;
        bot_msg.text = error.text;
        bot_msg.action = error.action;
    } else {
        drop(all_sessions);
        append_chat_bot_text(chat_state, session_id.clone(), error.text);
        return;
    }
    drop(all_sessions);
    if chat_state.active_session_id.read().as_deref() == Some(&session_id) {
        let messages = chat_state
            .session_messages
            .read()
            .get(&session_id)
            .cloned()
            .unwrap_or_default();
        chat_state.messages.set(messages);
    }
}
