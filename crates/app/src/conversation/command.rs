use std::pin::Pin;
use std::sync::Arc;

use domain::UserId;
use futures_core::Stream;

use crate::app_error::{AppError, AppResult};
use crate::conversation::ConversationAgentPort;

#[derive(Debug, Clone)]
pub struct SendUserMessageCommand {
    pub user_id: UserId,
    pub session_id: String,
    pub input: ConversationUserInput,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConversationUserInput {
    pub text: String,
    #[serde(default)]
    pub attachments: Vec<ConversationAttachment>,
}

impl ConversationUserInput {
    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            attachments: Vec::new(),
        }
    }

    pub fn has_images(&self) -> bool {
        self.attachments
            .iter()
            .any(|attachment| matches!(attachment, ConversationAttachment::Image(_)))
    }

    pub fn image_attachments(&self) -> impl Iterator<Item = &ConversationImageAttachment> {
        self.attachments.iter().filter_map(|attachment| {
            let ConversationAttachment::Image(image) = attachment;
            Some(image)
        })
    }

    pub fn visible_text(&self) -> String {
        let text = self.text.trim();
        if self.has_images() {
            if text.is_empty() {
                "[用户发送了一张图片]".to_string()
            } else {
                format!("{text}\n[用户发送了一张图片]")
            }
        } else {
            text.to_string()
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationAttachment {
    Image(ConversationImageAttachment),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationImageAttachment {
    pub asset_id: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct StoreEphemeralImageInput {
    pub user_id: UserId,
    pub session_id: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEphemeralImage {
    pub asset_id: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct EphemeralImageData {
    pub asset_id: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct SendUserMessageResult {
    pub reply: String,
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct ContinueInteractionCommand {
    pub user_id: UserId,
    pub session_id: String,
    pub interaction: AgentInteractionContinuation,
}

#[derive(Debug, Clone)]
pub enum AgentInteractionContinuation {
    ConfirmMealLog {
        pending_id: String,
    },
    RejectMealLog {
        pending_id: String,
    },
    SummarizeMealDay {
        finalized_at: String,
        meals_json: String,
    },
}

pub type ConversationReplyStream = Pin<Box<dyn Stream<Item = Result<String, AppError>> + Send>>;

#[derive(Clone)]
pub struct ConversationCommandHandler {
    agent: Arc<dyn ConversationAgentPort>,
}

impl ConversationCommandHandler {
    pub fn new(agent: Arc<dyn ConversationAgentPort>) -> Self {
        Self { agent }
    }

    pub async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult> {
        self.agent.send_user_message(command).await
    }

    pub async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.agent.stream_user_message(command).await
    }

    pub async fn continue_interaction(
        &self,
        command: ContinueInteractionCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.agent.continue_interaction(command).await
    }
}
