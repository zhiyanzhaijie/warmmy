use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

use app::conversation::{
    ChatMessage, ChatMessageAttachment, ChatMessageRepositoryPort, SaveMessageImageAttachment,
};
use domain::UserId;

use crate::persistence::sqlite::models::{ChatMessageAttachmentRow, ChatMessageRow};

const RIG_MEMORY_ROLE: &str = "rig_memory";

#[derive(Clone)]
pub struct SqliteChatMessageRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteChatMessageRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    async fn save_message_inner(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Option<String>, String> {
        if is_internal_conversation_message(role, content) {
            return Ok(None);
        }

        let mut db = self.db.lock().await;
        let created_at = Utc::now().to_rfc3339();
        let row = toasty::create!(ChatMessageRow {
            user_id: user_id.as_str().to_string(),
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at,
        })
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        Ok(Some(row.id.to_string()))
    }
}

#[async_trait]
impl ChatMessageRepositoryPort for SqliteChatMessageRepo {
    async fn find_by_session(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<ChatMessage>, String> {
        let mut db = self.db.lock().await;
        let rows = ChatMessageRow::filter(
            ChatMessageRow::fields()
                .user_id()
                .eq(user_id.as_str())
                .and(ChatMessageRow::fields().session_id().eq(session_id)),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        let mut messages: Vec<_> = rows
            .into_iter()
            .map(|row| (row.created_at, row.id, row.role, row.content))
            .collect();
        messages.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let attachments = ChatMessageAttachmentRow::filter(
            ChatMessageAttachmentRow::fields()
                .user_id()
                .eq(user_id.as_str())
                .and(ChatMessageAttachmentRow::fields().session_id().eq(session_id)),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        Ok(messages
            .into_iter()
            .filter(|row| row.2 != RIG_MEMORY_ROLE)
            .filter(|row| !is_internal_conversation_message(&row.2, &row.3))
            .map(|row| ChatMessage {
                id: row.1.to_string(),
                role: row.2,
                content: row.3,
                attachments: attachments_for_message(&attachments, &row.1.to_string()),
            })
            .collect())
    }

    async fn save_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Option<String>, String> {
        self.save_message_inner(user_id, session_id, role, content)
            .await
    }

    async fn save_message_with_attachments(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
        attachments: Vec<SaveMessageImageAttachment>,
    ) -> Result<Option<String>, String> {
        let Some(message_id) = self
            .save_message_inner(user_id, session_id, role, content)
            .await?
        else {
            return Ok(None);
        };

        let mut db = self.db.lock().await;
        let created_at = Utc::now().to_rfc3339();
        for (index, attachment) in attachments.into_iter().enumerate() {
            let width = attachment.width.and_then(|value| i32::try_from(value).ok());
            let height = attachment.height.and_then(|value| i32::try_from(value).ok());
            let size_bytes = i64::try_from(attachment.size_bytes).unwrap_or(i64::MAX);
            toasty::create!(ChatMessageAttachmentRow {
                id: format!("{message_id}:image:{index}"),
                user_id: user_id.as_str().to_string(),
                session_id: session_id.to_string(),
                message_id: message_id.clone(),
                kind: "image".to_string(),
                mime_type: attachment.mime_type,
                size_bytes,
                width,
                height,
                data_url: attachment.data_url,
                status: attachment.status,
                created_at: created_at.clone(),
            })
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())?;
        }

        Ok(Some(message_id))
    }

    async fn find_memory_messages(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<String>, String> {
        let mut db = self.db.lock().await;
        let rows = ChatMessageRow::filter(
            ChatMessageRow::fields()
                .user_id()
                .eq(user_id.as_str())
                .and(ChatMessageRow::fields().session_id().eq(session_id))
                .and(ChatMessageRow::fields().role().eq(RIG_MEMORY_ROLE)),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        let mut messages: Vec<_> = rows
            .into_iter()
            .map(|row| (row.created_at, row.id, row.content))
            .collect();
        messages.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        Ok(messages.into_iter().map(|row| row.2).collect())
    }

    async fn save_memory_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        content: &str,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let created_at = Utc::now().to_rfc3339();
        let _ = toasty::create!(ChatMessageRow {
            user_id: user_id.as_str().to_string(),
            session_id: session_id.to_string(),
            role: RIG_MEMORY_ROLE.to_string(),
            content: content.to_string(),
            created_at,
        })
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        Ok(())
    }

    async fn find_sessions(&self, user_id: &UserId) -> Result<Vec<String>, String> {
        let mut db = self.db.lock().await;
        // 查询该用户的所有消息，在内存中按创建时间排序，并去重 session_id (即日期)
        let rows = ChatMessageRow::filter(ChatMessageRow::fields().user_id().eq(user_id.as_str()))
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())?;

        let mut sessions: Vec<(String, String)> = rows
            .into_iter()
            .filter(|row| row.role != RIG_MEMORY_ROLE)
            .map(|row| (row.session_id, row.created_at))
            .collect();

        // 按创建时间倒序
        sessions.sort_by(|a, b| b.1.cmp(&a.1));

        let mut unique_sessions = Vec::new();
        for (sid, _) in sessions {
            if !unique_sessions.contains(&sid) {
                unique_sessions.push(sid);
            }
        }

        Ok(unique_sessions)
    }
}

fn is_internal_conversation_message(role: &str, content: &str) -> bool {
    role == "user"
        && (content
            .trim_start()
            .starts_with("[warmmy:internal-continuation]")
            || content.starts_with("用户已在界面确认一条待确认用餐记录。")
            || content.starts_with("用户已在界面取消一条待确认用餐记录。"))
}

fn attachments_for_message(
    rows: &[ChatMessageAttachmentRow],
    message_id: &str,
) -> Vec<ChatMessageAttachment> {
    rows.iter()
        .filter(|row| row.message_id == message_id)
        .map(|row| ChatMessageAttachment {
            id: row.id.clone(),
            kind: row.kind.clone(),
            mime_type: row.mime_type.clone(),
            size_bytes: u64::try_from(row.size_bytes).unwrap_or_default(),
            width: row.width.and_then(|value| u32::try_from(value).ok()),
            height: row.height.and_then(|value| u32::try_from(value).ok()),
            data_url: row.data_url.clone(),
            status: row.status.clone(),
        })
        .collect()
}
