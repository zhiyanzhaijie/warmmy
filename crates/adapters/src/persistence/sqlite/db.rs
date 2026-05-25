use toasty::Db;

use super::models::{
    ChatMessageRow, FoodNutritionReferenceRow, MealRecordRow, UserHealthExpectationRow,
    UserPreferencesRow, UserProfileRow,
};

pub async fn connect_sqlite(database_url: &str) -> toasty::Result<Db> {
    let db = toasty::Db::builder()
        .models(toasty::models!(
            UserProfileRow,
            UserHealthExpectationRow,
            UserPreferencesRow,
            MealRecordRow,
            FoodNutritionReferenceRow,
            ChatMessageRow
        ))
        .connect(database_url)
        .await?;
    if let Err(err) = db.push_schema().await {
        let message = err.to_string();
        if !message.contains("already exists") {
            return Err(err);
        }

        ensure_user_extension_tables(database_url)
            .map_err(toasty::Error::driver_operation_failed)?;
    }
    Ok(db)
}

fn ensure_user_extension_tables(database_url: &str) -> Result<(), rusqlite::Error> {
    let Some(path) = sqlite_path_from_url(database_url) else {
        return Ok(());
    };

    let connection = rusqlite::Connection::open(path)?;

    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS user_health_expectation_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT NOT NULL,
            kind TEXT NOT NULL,
            status TEXT NOT NULL,
            source_json TEXT NOT NULL,
            priority INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_user_health_expectation_rows_user_id
            ON user_health_expectation_rows (user_id);

        CREATE TABLE IF NOT EXISTS user_preferences_rows (
            user_id TEXT PRIMARY KEY NOT NULL,
            app_preferences_json TEXT NOT NULL,
            dietary_preferences_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS food_nutrition_reference_rows (
            id TEXT PRIMARY KEY NOT NULL,
            reference_id TEXT NOT NULL,
            labels_json TEXT NOT NULL,
            aliases_json TEXT NOT NULL,
            basis_quantity REAL NOT NULL,
            basis_unit TEXT NOT NULL,
            nutrition_json TEXT NOT NULL
        );
        "#,
    )?;

    ensure_column(
        &connection,
        "food_nutrition_reference_rows",
        "reference_id",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        &connection,
        "food_nutrition_reference_rows",
        "labels_json",
        "TEXT NOT NULL DEFAULT '{}' ",
    )?;

    Ok(())
}

fn ensure_column(
    connection: &rusqlite::Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), rusqlite::Error> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    for existing in columns {
        if existing? == column {
            return Ok(());
        }
    }

    connection.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
        [],
    )?;
    Ok(())
}

fn sqlite_path_from_url(database_url: &str) -> Option<&str> {
    if database_url == "sqlite::memory:" {
        return None;
    }

    if let Some(path) = database_url.strip_prefix("sqlite://") {
        return Some(path);
    }

    if let Some(path) = database_url.strip_prefix("sqlite:") {
        return Some(path);
    }

    Some(database_url)
}
