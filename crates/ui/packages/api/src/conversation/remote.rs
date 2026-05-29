use crate::impls::error::api_error;
use crate::impls::state::State;
use dioxus::fullstack::payloads::TextStream;
use dioxus::prelude::*;
use futures_util::StreamExt;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EchoResponse {
    pub reply: String,
    pub session_id: String,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct ChatSendInput {
    pub text: String,
    #[serde(default)]
    pub attachments: Vec<ChatImageAttachmentInput>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChatImageAttachmentInput {
    pub asset_id: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub preview_data_url: Option<String>,
}

#[post("/api/echo", state: State)]
pub async fn echo(
    user_id: String,
    input: ChatSendInput,
    session_id: String,
) -> Result<EchoResponse, ServerFnError> {
    let command = app::conversation::SendUserMessageCommand {
        user_id: parse_user_id(&user_id)?,
        session_id,
        input: to_app_input(input),
    };

    let result = state
        .0
        .conversation
        .command
        .send_user_message(command)
        .await
        .map_err(api_error)?;
    Ok(EchoResponse {
        reply: result.reply,
        session_id: result.session_id,
    })
}

#[post("/api/echo_stream", state: State)]
pub async fn echo_stream(
    user_id: String,
    input: ChatSendInput,
    session_id: String,
) -> Result<TextStream, ServerFnError> {
    let command = app::conversation::SendUserMessageCommand {
        user_id: parse_user_id(&user_id)?,
        session_id,
        input: to_app_input(input),
    };

    let stream = state
        .0
        .conversation
        .command
        .stream_user_message(command)
        .await
        .map_err(api_error)?;
    Ok(TextStream::new(stream.map(|item| match item {
        Ok(chunk) => chunk,
        Err(err) => format!("\n[stream error] {err}"),
    })))
}

#[post("/api/get_session_history", state: State)]
pub async fn get_session_history(
    user_id: String,
    session_id: String,
) -> Result<Vec<app::conversation::ChatMessage>, ServerFnError> {
    let user_id = parse_user_id(&user_id)?;
    let result = state
        .0
        .conversation
        .query
        .get_session_history(&user_id, &session_id)
        .await
        .map_err(api_error)?;
    Ok(result)
}

#[post("/api/store_ephemeral_image", state: State)]
pub async fn store_ephemeral_image(
    user_id: String,
    session_id: String,
    mime_type: String,
    bytes: Vec<u8>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<app::conversation::StoredEphemeralImage, ServerFnError> {
    let user_id = parse_user_id(&user_id)?;
    state
        .0
        .conversation
        .image_store
        .put_image(app::conversation::StoreEphemeralImageInput {
            user_id,
            session_id,
            mime_type,
            bytes,
            width,
            height,
        })
        .await
        .map_err(api_error)
}

#[post("/api/delete_ephemeral_image", state: State)]
pub async fn delete_ephemeral_image(asset_id: String) -> Result<(), ServerFnError> {
    state
        .0
        .conversation
        .image_store
        .delete_image(&asset_id)
        .await
        .map_err(api_error)
}

#[post("/api/list_user_sessions", state: State)]
pub async fn list_user_sessions(user_id: String) -> Result<Vec<String>, ServerFnError> {
    let user_id = parse_user_id(&user_id)?;
    let result = state
        .0
        .conversation
        .query
        .list_user_sessions(&user_id)
        .await
        .map_err(api_error)?;
    Ok(result)
}

fn parse_user_id(value: &str) -> Result<domain::UserId, ServerFnError> {
    domain::UserId::parse(value)
        .map_err(|err| ServerFnError::new(format!("invalid user_id `{}`: {}", value.trim(), err)))
}

fn to_app_input(input: ChatSendInput) -> app::conversation::ConversationUserInput {
    app::conversation::ConversationUserInput {
        text: input.text,
        attachments: input
            .attachments
            .into_iter()
            .map(|image| {
                app::conversation::ConversationAttachment::Image(
                    app::conversation::ConversationImageAttachment {
                        asset_id: image.asset_id,
                        mime_type: image.mime_type,
                        size_bytes: image.size_bytes,
                        width: image.width,
                        height: image.height,
                        preview_data_url: image.preview_data_url,
                    },
                )
            })
            .collect(),
    }
}
