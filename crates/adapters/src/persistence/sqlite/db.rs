use toasty::Db;

use super::models::{MealRecordRow, UserProfileRow};

pub async fn connect_sqlite(database_url: &str) -> toasty::Result<Db> {
    let db = toasty::Db::builder()
        .models(toasty::models!(UserProfileRow, MealRecordRow))
        .connect(database_url)
        .await?;
    db.push_schema().await?;
    Ok(db)
}
