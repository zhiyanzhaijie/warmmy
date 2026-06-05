use toasty::Db;

use super::models::{
    ChatMessageAttachmentRow, ChatMessageRow, ChatSummaryRow, DiningCompanionRow,
    FoodNutritionReferenceRow, MealDayFinalizationRow, MealDaySummaryRow, MealRecordRow,
    MemoryRecordRow, PendingMealLogRow, UserAIProviderRow, UserAIRouteRow,
    UserHealthExpectationRow, UserPreferencesRow, UserProfileRow, UserSecretRow,
};

pub async fn connect_sqlite(database_url: &str) -> toasty::Result<Db> {
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
