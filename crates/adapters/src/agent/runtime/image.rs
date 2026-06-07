use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::conversation::{
    ChatMessageRepositoryPort, ConversationUserInput, EphemeralImageData, EphemeralImageStorePort,
    SaveMessageImageAttachment,
};
use base64::Engine;
use domain::UserId;
use rig::OneOrMany;
use rig::completion::Message;
use rig::message::{ImageDetail, ImageMediaType, MimeType, UserContent};

pub async fn build_prompt_message(
    image_store: &Arc<dyn EphemeralImageStorePort>,
    input: &ConversationUserInput,
) -> AppResult<Message> {
    let images = load_image_data(image_store, input).await?;
    let mut content = Vec::new();
    let text = input.text.trim();

    if !text.is_empty() || images.is_empty() {
        content.push(UserContent::text(text.to_string()));
    }

    for image in images {
        let media_type = ImageMediaType::from_mime_type(&image.mime_type).ok_or_else(|| {
            AppError::validation(format!("unsupported image mime type: {}", image.mime_type))
        })?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(image.bytes);
        content.push(UserContent::image_base64(
            encoded,
            Some(media_type),
            Some(ImageDetail::Auto),
        ));
    }

    let content =
        OneOrMany::many(content).map_err(|_| AppError::validation("empty conversation input"))?;
    Ok(Message::User { content })
}

pub async fn persist_user_image_message(
    repo: &Arc<dyn ChatMessageRepositoryPort>,
    user_id: &UserId,
    session_id: &str,
    input: &ConversationUserInput,
    visible_text: &str,
) {
    let attachments = input
        .image_attachments()
        .map(|image| SaveMessageImageAttachment {
            mime_type: image.mime_type.clone(),
            size_bytes: image.size_bytes,
            width: image.width,
            height: image.height,
            data_url: image.preview_data_url.clone(),
            status: if image.preview_data_url.is_some() {
                "available".to_string()
            } else {
                "missing".to_string()
            },
        })
        .collect::<Vec<_>>();

    if attachments.is_empty() {
        return;
    }

    if let Err(err) = repo
        .save_message_with_attachments(user_id, session_id, "user", visible_text, attachments)
        .await
    {
        tracing::warn!(error = %err, "failed to persist image message attachments");
    }
}

async fn load_image_data(
    image_store: &Arc<dyn EphemeralImageStorePort>,
    input: &ConversationUserInput,
) -> AppResult<Vec<EphemeralImageData>> {
    let mut images = Vec::new();
    for image in input.image_attachments() {
        images.push(image_store.load_image(&image.asset_id).await?);
    }
    Ok(images)
}
