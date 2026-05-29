use std::sync::Arc;

use app::conversation::ChatMessageRepositoryPort;
use domain::UserId;
use rig::memory::{ConversationMemory, MemoryError};
use rig::message::{AssistantContent, Message, UserContent};
use rig::wasm_compat::WasmBoxedFuture;

const INTERNAL_CONVERSATION_MARKER: &str = "[warmmy:internal-continuation]";

#[derive(Clone)]
pub struct SessionConversationMemory {
    user_id: UserId,
    repo: Arc<dyn ChatMessageRepositoryPort>,
    max_recent_messages: usize,
}

impl SessionConversationMemory {
    pub fn new(
        user_id: UserId,
        repo: Arc<dyn ChatMessageRepositoryPort>,
        max_recent_messages: usize,
    ) -> Self {
        Self {
            user_id,
            repo,
            max_recent_messages,
        }
    }
}

impl ConversationMemory for SessionConversationMemory {
    fn load<'a>(
        &'a self,
        conversation_id: &'a str,
    ) -> WasmBoxedFuture<'a, Result<Vec<Message>, MemoryError>> {
        Box::pin(async move {
            let memory_messages = self
                .repo
                .find_memory_messages(&self.user_id, conversation_id)
                .await
                .map_err(|err| MemoryError::Policy(err.to_string()))?;

            if !memory_messages.is_empty() {
                let mut history = memory_messages
                    .into_iter()
                    .filter_map(|content| serde_json::from_str::<Message>(&content).ok())
                    .collect::<Vec<_>>();
                apply_recent_window(&mut history, self.max_recent_messages);
                return Ok(history);
            }

            let messages = self
                .repo
                .find_by_session(&self.user_id, conversation_id)
                .await
                .map_err(|err| MemoryError::Policy(err.to_string()))?;

            let mut history: Vec<Message> = messages
                .into_iter()
                .filter_map(|msg| match msg.role.as_str() {
                    "user" => Some(Message::user(msg.content)),
                    "assistant" => Some(Message::assistant(msg.content)),
                    _ => None,
                })
                .collect();

            apply_recent_window(&mut history, self.max_recent_messages);
            Ok(history)
        })
    }

    fn append<'a>(
        &'a self,
        conversation_id: &'a str,
        messages: Vec<Message>,
    ) -> WasmBoxedFuture<'a, Result<(), MemoryError>> {
        Box::pin(async move {
            for message in messages {
                if !is_internal_message(&message) {
                    let raw = serde_json::to_string(&message)
                        .map_err(|err| MemoryError::Policy(err.to_string()))?;
                    self.repo
                        .save_memory_message(&self.user_id, conversation_id, &raw)
                        .await
                        .map_err(|err| MemoryError::Policy(err.to_string()))?;
                }

                if let Some((role, content)) = visible_text_message(&message) {
                    if role == "user" && content.contains("[用户发送了一张图片]") {
                        continue;
                    }
                    self.repo
                        .save_message(&self.user_id, conversation_id, role, &content)
                        .await
                        .map_err(|err| MemoryError::Policy(err.to_string()))?;
                }
            }

            Ok(())
        })
    }

    fn clear<'a>(
        &'a self,
        _conversation_id: &'a str,
    ) -> WasmBoxedFuture<'a, Result<(), MemoryError>> {
        Box::pin(async move { Ok(()) })
    }
}

fn apply_recent_window(history: &mut Vec<Message>, max_recent_messages: usize) {
    if max_recent_messages == 0 || history.len() <= max_recent_messages {
        return;
    }

    let keep_from = history.len() - max_recent_messages;
    history.drain(0..keep_from);
}

fn visible_text_message(message: &Message) -> Option<(&'static str, String)> {
    match message {
        Message::User { content } => {
            let text = content
                .iter()
                .filter_map(|item| {
                    if let UserContent::Text(text) = item {
                        Some(text.text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");

            if is_internal_continuation(&text) {
                return None;
            }

            (!text.is_empty()).then_some(("user", text))
        }
        Message::Assistant { content, .. } => {
            let text = content
                .iter()
                .filter_map(|item| {
                    if let AssistantContent::Text(text) = item {
                        Some(text.text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");

            (!text.is_empty()).then_some(("assistant", text))
        }
        Message::System { .. } => None,
    }
}

fn is_internal_message(message: &Message) -> bool {
    match message {
        Message::User { content } => content.iter().any(|item| {
            if let UserContent::Text(text) = item {
                is_internal_continuation(&text.text)
            } else {
                false
            }
        }),
        _ => false,
    }
}

fn is_internal_continuation(text: &str) -> bool {
    text.trim_start().starts_with(INTERNAL_CONVERSATION_MARKER)
        || text.starts_with("用户已在界面确认一条待确认用餐记录。")
        || text.starts_with("用户已在界面取消一条待确认用餐记录。")
}
