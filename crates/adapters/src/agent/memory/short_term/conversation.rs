use std::sync::Arc;

use app::conversation::{ChatMessage, ChatMessageRepositoryPort};
use domain::UserId;
use rig::memory::{ConversationMemory, MemoryError};
use rig::message::Message;
use rig::wasm_compat::WasmBoxedFuture;

const CONVERSATION_SUMMARY_PREFIX: &str = "Conversation summary for earlier messages in this session:";

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
            let summary = self
                .repo
                .find_conversation_summary(&self.user_id, conversation_id)
                .await
                .map_err(|err| MemoryError::Policy(err.to_string()))?;

            let visible_messages = self
                .repo
                .find_by_session(&self.user_id, conversation_id)
                .await
                .map_err(|err| MemoryError::Policy(err.to_string()))?;

            let mut history = Vec::new();
            if let Some(summary) = summary {
                if !summary.summary.trim().is_empty() {
                    history.push(Message::System {
                        content: format!("{CONVERSATION_SUMMARY_PREFIX}\n{}", summary.summary),
                    });
                }
            }

            history.extend(
                recent_visible_messages(visible_messages, self.max_recent_messages)
                    .into_iter()
                    .filter_map(message_from_chat_message),
            );
            Ok(history)
        })
    }

    fn append<'a>(
        &'a self,
        conversation_id: &'a str,
        messages: Vec<Message>,
    ) -> WasmBoxedFuture<'a, Result<(), MemoryError>> {
        let _ = (conversation_id, messages);
        Box::pin(async move { Ok(()) })
    }

    fn clear<'a>(
        &'a self,
        _conversation_id: &'a str,
    ) -> WasmBoxedFuture<'a, Result<(), MemoryError>> {
        Box::pin(async move { Ok(()) })
    }
}

fn recent_visible_messages(
    mut messages: Vec<ChatMessage>,
    max_recent_messages: usize,
) -> Vec<ChatMessage> {
    if max_recent_messages == 0 || messages.len() <= max_recent_messages {
        return messages;
    }
    messages.split_off(messages.len() - max_recent_messages)
}

fn message_from_chat_message(message: ChatMessage) -> Option<Message> {
    if !message.attachments.is_empty() || message.content.trim().is_empty() {
        return None;
    }

    match message.role.as_str() {
        "user" => Some(Message::user(message.content)),
        "assistant" => Some(Message::assistant(message.content)),
        _ => None,
    }
}
