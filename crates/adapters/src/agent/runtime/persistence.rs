use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::conversation::{ChatMessageRepositoryPort, ConversationUserInput};
use domain::UserId;

const INTERNAL_CONVERSATION_MARKER: &str = "[warmmy:internal-continuation]";

pub async fn persist_user_visible_message(
    repo: &Arc<dyn ChatMessageRepositoryPort>,
    user_id: &UserId,
    session_id: &str,
    input: &ConversationUserInput,
    visible_text: &str,
) -> AppResult<()> {
    let visible_text = visible_text.trim();
    if visible_text.is_empty() || input.has_images() || is_internal_conversation_input(visible_text)
    {
        return Ok(());
    }

    repo.save_message(user_id, session_id, "user", visible_text)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub async fn persist_assistant_visible_message(
    repo: &Arc<dyn ChatMessageRepositoryPort>,
    user_id: &UserId,
    session_id: &str,
    reply: &str,
) -> AppResult<()> {
    let reply = reply.trim();
    if reply.is_empty() {
        return Ok(());
    }

    repo.save_message(user_id, session_id, "assistant", reply)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub fn is_internal_conversation_input(text: &str) -> bool {
    text.trim_start().starts_with(INTERNAL_CONVERSATION_MARKER)
        || text.starts_with("用户已在界面确认一条待确认用餐记录。")
        || text.starts_with("用户已在界面取消一条待确认用餐记录。")
}
