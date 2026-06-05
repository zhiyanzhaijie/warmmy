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



        CREATE TABLE IF NOT EXISTS user_ai_provider_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            base_url TEXT NOT NULL,
            secret_ref TEXT,
            enabled BOOLEAN NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_user_ai_provider_rows_user_id
            ON user_ai_provider_rows (user_id);

        CREATE TABLE IF NOT EXISTS user_ai_route_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            capability TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            model TEXT NOT NULL,
            embedding_ndims INTEGER,
            enabled BOOLEAN NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_user_ai_route_rows_user_id
            ON user_ai_route_rows (user_id);

        CREATE TABLE IF NOT EXISTS user_secret_rows (
            id TEXT PRIMARY KEY NOT NULL,
            scope TEXT NOT NULL,
            secret_value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_user_secret_rows_scope
            ON user_secret_rows (scope);

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
            name TEXT NOT NULL,
            terms_json TEXT NOT NULL,
            basis_quantity REAL NOT NULL,
            basis_unit TEXT NOT NULL,
            nutrition_json TEXT NOT NULL,
            status TEXT NOT NULL,
            source TEXT,
            confidence REAL
        );

        CREATE TABLE IF NOT EXISTS chat_message_attachment_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            message_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            width INTEGER,
            height INTEGER,
            data_url TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_chat_message_attachment_rows_user_id
            ON chat_message_attachment_rows (user_id);
        CREATE INDEX IF NOT EXISTS idx_chat_message_attachment_rows_session_id
            ON chat_message_attachment_rows (session_id);
        CREATE INDEX IF NOT EXISTS idx_chat_message_attachment_rows_message_id
            ON chat_message_attachment_rows (message_id);

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

        CREATE TABLE IF NOT EXISTS chat_summary_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            summarized_until_message_id TEXT,
            summarized_until_index INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_chat_summary_rows_user_session
            ON chat_summary_rows (user_id, session_id);
        CREATE INDEX IF NOT EXISTS idx_chat_summary_rows_user_id
            ON chat_summary_rows (user_id);
        CREATE INDEX IF NOT EXISTS idx_chat_summary_rows_session_id
            ON chat_summary_rows (session_id);

        CREATE TABLE IF NOT EXISTS memory_record_rows (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            tier TEXT NOT NULL,
            form TEXT,
            scope_json TEXT NOT NULL,
            source_json TEXT NOT NULL,
            status TEXT NOT NULL,
            index_policy TEXT NOT NULL,
            content TEXT NOT NULL,
            search_text TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0,
            claim_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_memory_record_rows_user_id
            ON memory_record_rows (user_id);
        CREATE INDEX IF NOT EXISTS idx_memory_record_rows_status
            ON memory_record_rows (status);
        CREATE INDEX IF NOT EXISTS idx_memory_record_rows_index_policy
            ON memory_record_rows (index_policy);
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
        "name",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        &connection,
        "food_nutrition_reference_rows",
        "terms_json",
        "TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        &connection,
        "food_nutrition_reference_rows",
        "status",
        "TEXT NOT NULL DEFAULT 'seed'",
    )?;
    ensure_column(
        &connection,
        "food_nutrition_reference_rows",
        "source",
        "TEXT DEFAULT NULL",
    )?;
    ensure_column(
        &connection,
        "food_nutrition_reference_rows",
        "confidence",
        "REAL DEFAULT NULL",
    )?;
    rebuild_food_nutrition_reference_table_if_legacy(&connection)?;

    ensure_column(
        &connection,
        "memory_record_rows",
        "confidence",
        "REAL NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        &connection,
        "memory_record_rows",
        "claim_json",
        "TEXT DEFAULT NULL",
    )?;

    Ok(())
}

fn rebuild_food_nutrition_reference_table_if_legacy(
    connection: &rusqlite::Connection,
) -> Result<(), rusqlite::Error> {
    let columns = table_columns(connection, "food_nutrition_reference_rows")?;
    let has_legacy_columns = ["reference_id", "labels_json", "aliases_json"]
        .iter()
        .any(|column| columns.iter().any(|existing| existing == column));
    if !has_legacy_columns {
        return Ok(());
    }

    connection.execute_batch(
        r#"
        BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS food_nutrition_reference_rows_new (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            terms_json TEXT NOT NULL,
            basis_quantity REAL NOT NULL,
            basis_unit TEXT NOT NULL,
            nutrition_json TEXT NOT NULL,
            status TEXT NOT NULL,
            source TEXT,
            confidence REAL
        );
        INSERT OR REPLACE INTO food_nutrition_reference_rows_new (
            id,
            name,
            terms_json,
            basis_quantity,
            basis_unit,
            nutrition_json,
            status,
            source,
            confidence
        )
        SELECT
            id,
            name,
            terms_json,
            basis_quantity,
            basis_unit,
            nutrition_json,
            status,
            source,
            confidence
        FROM food_nutrition_reference_rows
        WHERE id <> ''
          AND name <> ''
          AND terms_json <> ''
          AND nutrition_json <> '';
        DROP TABLE food_nutrition_reference_rows;
        ALTER TABLE food_nutrition_reference_rows_new RENAME TO food_nutrition_reference_rows;
        COMMIT;
        "#,
    )?;

    Ok(())
}

fn table_columns(
    connection: &rusqlite::Connection,
    table: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let mut columns = Vec::new();
    for row in rows {
        columns.push(row?);
    }
    Ok(columns)
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
