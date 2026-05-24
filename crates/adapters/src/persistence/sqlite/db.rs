use toasty::Db;

use super::models::{ChatMessageRow, MealRecordRow, UserProfileRow};

pub async fn connect_sqlite(database_url: &str) -> toasty::Result<Db> {
    let db = toasty::Db::builder()
        .models(toasty::models!(
            UserProfileRow,
            MealRecordRow,
            ChatMessageRow
        ))
        .connect(database_url)
        .await?;
    if let Err(err) = db.push_schema().await {
        let message = err.to_string();
        if !message.contains("already exists") {
            return Err(err);
        }
    }
    Ok(db)
}
