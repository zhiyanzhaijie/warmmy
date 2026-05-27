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

#[post("/api/echo", state: State)]
pub async fn echo(
    user_id: String,
    input: String,
    session_id: String,
) -> Result<EchoResponse, ServerFnError> {
    let command = app::conversation::SendUserMessageCommand {
        user_id: parse_user_id(&user_id)?,
        session_id,
        content: input,
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
    input: String,
    session_id: String,
) -> Result<TextStream, ServerFnError> {
    let command = app::conversation::SendUserMessageCommand {
        user_id: parse_user_id(&user_id)?,
        session_id,
        content: input,
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
