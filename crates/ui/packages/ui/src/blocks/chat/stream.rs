use dioxus::prelude::*;
use serde_json::Value;

use super::state::{
    ChatMessage, ChatMessageAttachment, ComposerImageAttachment, ACTIVE_SESSION_ID, CHAT_MESSAGES,
    CHAT_NEXT_ID,
};
use api::meal;

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

pub fn next_chat_id() -> u64 {
    CHAT_MESSAGES
        .read()
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

pub fn is_active_session(session_id: &str) -> bool {
    ACTIVE_SESSION_ID
        .read()
        .as_ref()
        .map(|active| active == session_id)
        .unwrap_or(false)
}

pub fn append_pending_meal_messages(pending_meals: Vec<meal::PendingMealLogDTO>, start_id: u64) {
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
            attachments: Vec::new(),
            pending_meal: Some(pending_meal),
        });
        next_id += 1;
    }
    *CHAT_NEXT_ID.write() = next_id;
}

pub fn append_outgoing_message_pair(
    content: String,
    attachments: Vec<ComposerImageAttachment>,
) -> u64 {
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
        attachments: composer_attachments_to_message_attachments(attachments),
        pending_meal: None,
    });
    messages.push(ChatMessage {
        id: bot_id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        attachments: Vec::new(),
        pending_meal: None,
    });
    bot_id
}

pub fn append_bot_text(text: String) {
    let id = CHAT_NEXT_ID();
    *CHAT_NEXT_ID.write() = id.saturating_add(1);
    CHAT_MESSAGES.write().push(ChatMessage {
        id,
        text,
        is_bot: true,
        is_skeleton: false,
        is_streaming: false,
        attachments: Vec::new(),
        pending_meal: None,
    });
}

pub fn append_streaming_bot_slot() -> u64 {
    let id = CHAT_NEXT_ID();
    *CHAT_NEXT_ID.write() = id.saturating_add(1);
    CHAT_MESSAGES.write().push(ChatMessage {
        id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        attachments: Vec::new(),
        pending_meal: None,
    });
    id
}

pub async fn append_agent_stream(
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
        attachments: Vec::new(),
        pending_meal: Some(pending_meal),
    });
}

fn composer_attachments_to_message_attachments(
    attachments: Vec<ComposerImageAttachment>,
) -> Vec<ChatMessageAttachment> {
    attachments
        .into_iter()
        .map(|attachment| ChatMessageAttachment {
            id: format!("composer:{}", attachment.id),
            kind: "image".to_string(),
            mime_type: attachment.mime_type,
            size_bytes: attachment.size_bytes,
            width: None,
            height: None,
            data_url: Some(attachment.preview_data_url),
            status: "available".to_string(),
        })
        .collect()
}
