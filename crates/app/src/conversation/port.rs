use async_trait::async_trait;
use domain::UserId;

use crate::app_error::AppResult;
use crate::conversation::{
    ChatMessage, ContinueInteractionCommand, ConversationReplyStream, EphemeralImageData,
    SaveMessageImageAttachment, SendUserMessageCommand, SendUserMessageResult,
    StoreEphemeralImageInput, StoredEphemeralImage,
};

#[async_trait]
pub trait ConversationAgentPort: Send + Sync {
    async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult>;

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream>;

    async fn continue_interaction(
        &self,
        command: ContinueInteractionCommand,
    ) -> AppResult<ConversationReplyStream>;
}

#[async_trait]
pub trait ChatMessageRepositoryPort: Send + Sync {
    async fn find_by_session(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<ChatMessage>, String>;
    async fn save_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Option<String>, String>;
    async fn save_message_with_attachments(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
        attachments: Vec<SaveMessageImageAttachment>,
    ) -> Result<Option<String>, String>;
    async fn find_memory_messages(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<String>, String>;
    async fn save_memory_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        content: &str,
    ) -> Result<(), String>;
    async fn find_sessions(&self, user_id: &UserId) -> Result<Vec<String>, String>;
}

#[async_trait]
pub trait EphemeralImageStorePort: Send + Sync {
    async fn put_image(
        &self,
        input: StoreEphemeralImageInput,
    ) -> AppResult<StoredEphemeralImage>;

    async fn load_image(&self, asset_id: &str) -> AppResult<EphemeralImageData>;

    async fn delete_image(&self, asset_id: &str) -> AppResult<()>;

    async fn cleanup_expired(&self) -> AppResult<()>;
}
