use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

use app::conversation::{ChatMessage, ChatMessageRepositoryPort};
use domain::UserId;

use crate::persistence::sqlite::models::ChatMessageRow;

#[derive(Clone)]
pub struct SqliteChatMessageRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteChatMessageRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
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

        Ok(messages
            .into_iter()
            .map(|row| ChatMessage {
                role: row.2,
                content: row.3,
            })
            .collect())
    }

    async fn save_message(
        &self,
        user_id: &UserId,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let created_at = Utc::now().to_rfc3339();
        let _ = toasty::create!(ChatMessageRow {
            user_id: user_id.as_str().to_string(),
            session_id: session_id.to_string(),
            role: role.to_string(),
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
