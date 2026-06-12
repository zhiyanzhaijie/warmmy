use std::path::{Path, PathBuf};

use toasty::Db;

use super::models::{
    ChatMessageAttachmentRow, ChatMessageRow, ChatSummaryRow, DiningCompanionRow,
    FoodNutritionReferenceRow, MealDayFinalizationRow, MealDaySummaryRow, MealRecordRow,
    MemoryRecordRow, PendingMealLogRow, UserAIProviderRow, UserAIRouteRow,
    UserHealthExpectationRow, UserPreferencesRow, UserProfileRow, UserSecretRow,
};

pub async fn connect_sqlite(database_url: &str) -> toasty::Result<Db> {
    let driver = sqlite_driver(database_url);
    let db = toasty::Db::builder()
        .models(toasty::models!(
            UserProfileRow,
            UserHealthExpectationRow,
            UserPreferencesRow,
            DiningCompanionRow,
            UserAIProviderRow,
            UserAIRouteRow,
            UserSecretRow,
            MealRecordRow,
            MealDayFinalizationRow,
            MealDaySummaryRow,
            PendingMealLogRow,
            FoodNutritionReferenceRow,
            ChatMessageRow,
            ChatMessageAttachmentRow,
            ChatSummaryRow,
            MemoryRecordRow
        ))
        .build(driver)
        .await?;
    if let Err(err) = db.push_schema().await {
        let message = err.to_string();
        if !message.contains("already exists") {
            return Err(err);
        }
    }
    Ok(db)
}

fn sqlite_driver(database_url: &str) -> toasty_driver_sqlite::Sqlite {
    if database_url == "sqlite::memory:" || database_url == "sqlite://memory" {
        return toasty_driver_sqlite::Sqlite::in_memory();
    }

    toasty_driver_sqlite::Sqlite::open(sqlite_database_path(database_url))
}

fn sqlite_database_path(database_url: &str) -> PathBuf {
    let path = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);

    Path::new(path).to_path_buf()
}
