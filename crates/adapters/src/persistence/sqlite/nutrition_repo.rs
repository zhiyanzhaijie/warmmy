use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SqliteNutritionRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteNutritionRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    pub async fn is_ready(&self) -> bool {
        let _db_guard = self.db.lock().await;
        true
    }
}
