use std::sync::Arc;

use app::conversation::ChatMessageRepositoryPort;
use domain::UserId;
use rig::memory::{ConversationMemory, MemoryError};
use rig::message::{AssistantContent, Message, UserContent};
use rig::wasm_compat::WasmBoxedFuture;

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
                if let Some((role, content)) = text_message(&message) {
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

fn text_message(message: &Message) -> Option<(&'static str, String)> {
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
