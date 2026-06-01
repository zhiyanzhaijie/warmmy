use dioxus::prelude::*;
use dioxus_sdk_time::sleep;
use futures_util::future::{select, Either};
use std::rc::Rc;
use std::time::Duration;

use api::conversation;

use crate::blocks::{
    activate_chat_session, append_agent_stream, append_chat_bot_text,
    append_outgoing_message_pair, ChatRuntimeContext, ChatStateContext, ComposerImageAttachment,
    ConversationTransitionContext, SendConversationMessage,
};

use super::current_user_id;

const CHAT_STREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(45);
const CHAT_FALLBACK_TIMEOUT: Duration = Duration::from_secs(45);

#[component]
pub fn ChatRuntimeProvider(children: Element) -> Element {
    let chat_state = use_context::<ChatStateContext>();
    let transition = use_context::<ConversationTransitionContext>();
    let user_id = current_user_id();
    let mut send_message = use_signal(|| None);

    use_context_provider(|| ChatRuntimeContext { send_message });

    use_effect(move || {
        let request_user_id = user_id.clone();
        let sender = SendConversationMessage::new(Rc::new(
            move |session_id: String,
                  content: String,
                  attachments: Vec<ComposerImageAttachment>,
                  route_after_stream: bool| {
                send_conversation_message(
                    chat_state,
                    transition,
                    request_user_id.clone(),
                    session_id,
                    content,
                    attachments,
                    route_after_stream,
                );
            },
        ));
        send_message.set(Some(sender));
    });

    rsx! {
        {children}
    }
}

fn send_conversation_message(
    chat_state: ChatStateContext,
    transition: ConversationTransitionContext,
    request_user_id: String,
    session_id: String,
    content: String,
    attachments: Vec<ComposerImageAttachment>,
    route_after_stream: bool,
) {
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

    dioxus::core::spawn_forever(async move {
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
                        format!("图片上传失败：{err}"),
                    );
                    clear_pending_transition(transition, &session_id);
                    return;
                }
            }
        }

        let send_input = conversation::ChatSendInput {
            text: content,
            attachments: uploaded_attachments,
        };
        match echo_stream_with_timeout(
            request_user_id.clone(),
            send_input.clone(),
            session_id.clone(),
        )
        .await
        {
            Ok(stream) => {
                append_agent_stream(chat_state, stream, bot_id, session_id.clone()).await;
                if route_after_stream {
                    navigator().replace(format!("/{session_id}"));
                }
                clear_pending_transition(transition, &session_id);
            }
            Err(stream_err) => {
                let fallback =
                    match echo_with_timeout(request_user_id, send_input, session_id.clone()).await {
                        Ok(resp) => resp.reply,
                        Err(err) => format!("对话请求失败：{stream_err}; fallback failed: {err}"),
                    };
                stop_bot_with_text(chat_state, session_id.clone(), bot_id, fallback);
                if route_after_stream {
                    navigator().replace(format!("/{session_id}"));
                }
                clear_pending_transition(transition, &session_id);
            }
        }
    });
}

async fn echo_stream_with_timeout(
    user_id: String,
    input: conversation::ChatSendInput,
    session_id: String,
) -> Result<dioxus::fullstack::payloads::TextStream, String> {
    let stream = conversation::echo_stream(user_id, input, session_id);
    let timeout = sleep(CHAT_STREAM_CONNECT_TIMEOUT);
    futures_util::pin_mut!(stream);
    futures_util::pin_mut!(timeout);

    match select(stream, timeout).await {
        Either::Left((result, _)) => result.map_err(|err| err.to_string()),
        Either::Right((_, _)) => Err(format!(
            "请求模型超时（{} 秒）。请检查手机网络、API base URL、模型名称和 API key。",
            CHAT_STREAM_CONNECT_TIMEOUT.as_secs()
        )),
    }
}

async fn echo_with_timeout(
    user_id: String,
    input: conversation::ChatSendInput,
    session_id: String,
) -> Result<conversation::EchoResponse, String> {
    let response = conversation::echo(user_id, input, session_id);
    let timeout = sleep(CHAT_FALLBACK_TIMEOUT);
    futures_util::pin_mut!(response);
    futures_util::pin_mut!(timeout);

    match select(response, timeout).await {
        Either::Left((result, _)) => result.map_err(|err| err.to_string()),
        Either::Right((_, _)) => Err(format!(
            "非流式 fallback 也超时（{} 秒）",
            CHAT_FALLBACK_TIMEOUT.as_secs()
        )),
    }
}

fn stop_bot_with_text(
    mut chat_state: ChatStateContext,
    session_id: String,
    bot_id: u64,
    text: String,
) {
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
        bot_msg.is_skeleton = false;
        bot_msg.is_streaming = false;
        bot_msg.text = text;
    } else {
        drop(all_sessions);
        append_chat_bot_text(chat_state, session_id.clone(), text);
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

fn clear_pending_transition(mut transition: ConversationTransitionContext, session_id: &str) {
    transition.pending.with_mut(|pending| {
        if pending
            .as_ref()
            .map(|pending| pending.session_id == session_id)
            .unwrap_or(false)
        {
            *pending = None;
        }
    });
}
