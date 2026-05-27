use toasty::Db;

use super::models::{
    ChatMessageRow, DiningCompanionRow, FoodNutritionReferenceRow, MealDayFinalizationRow,
    MealDaySummaryRow, MealRecordRow, PendingMealLogRow, UserHealthExpectationRow,
    UserPreferencesRow, UserProfileRow,
};

pub async fn connect_sqlite(database_url: &str) -> toasty::Result<Db> {
    let db = toasty::Db::builder()
        .models(toasty::models!(
            UserProfileRow,
            UserHealthExpectationRow,
            UserPreferencesRow,
            DiningCompanionRow,
            MealRecordRow,
            MealDayFinalizationRow,
            MealDaySummaryRow,
            PendingMealLogRow,
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
        CREATE TABLE IF NOT EXISTS user_profile_rows (
            id TEXT PRIMARY KEY NOT NULL,
            display_name TEXT NOT NULL,
            introduction TEXT NOT NULL,
            allergies_json TEXT NOT NULL,
            gender TEXT,
            age INTEGER
        );

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

        CREATE TABLE IF NOT EXISTS dining_companion_rows (
            id TEXT PRIMARY KEY NOT NULL,
            owner_user_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            relationship TEXT,
            introduction TEXT NOT NULL,
            dietary_preferences_json TEXT NOT NULL,
            health_notes_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_dining_companion_rows_owner_user_id
            ON dining_companion_rows (owner_user_id);

        CREATE TABLE IF NOT EXISTS food_nutrition_reference_rows (
            id TEXT PRIMARY KEY NOT NULL,
            reference_id TEXT NOT NULL,
            labels_json TEXT NOT NULL,
            aliases_json TEXT NOT NULL,
            basis_quantity REAL NOT NULL,
            basis_unit TEXT NOT NULL,
            nutrition_json TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS meal_day_finalization_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            finalized_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_meal_day_finalization_rows_user_id
            ON meal_day_finalization_rows (user_id);
        CREATE INDEX IF NOT EXISTS idx_meal_day_finalization_rows_session_id
            ON meal_day_finalization_rows (session_id);

        CREATE TABLE IF NOT EXISTS meal_day_summary_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            content TEXT NOT NULL,
            nutrition_score REAL NOT NULL,
            expectation_match_score REAL NOT NULL,
            overall_score REAL NOT NULL,
            metrics_json TEXT NOT NULL,
            finalized_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_meal_day_summary_rows_user_id
            ON meal_day_summary_rows (user_id);
        CREATE INDEX IF NOT EXISTS idx_meal_day_summary_rows_session_id
            ON meal_day_summary_rows (session_id);

        CREATE TABLE IF NOT EXISTS pending_meal_log_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            day_cycle TEXT NOT NULL,
            foods_json TEXT NOT NULL,
            nutrition_json TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_pending_meal_log_rows_user_id
            ON pending_meal_log_rows (user_id);
        CREATE INDEX IF NOT EXISTS idx_pending_meal_log_rows_session_id
            ON pending_meal_log_rows (session_id);
        "#,
    )?;

    ensure_column(
        &connection,
        "user_profile_rows",
        "gender",
        "TEXT DEFAULT NULL",
    )?;
    ensure_column(
        &connection,
        "user_profile_rows",
        "age",
        "INTEGER DEFAULT NULL",
    )?;

    ensure_column(
        &connection,
        "meal_record_rows",
        "session_id",
        "TEXT NOT NULL DEFAULT ''",
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
