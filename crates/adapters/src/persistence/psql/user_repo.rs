use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use application::user::UserProfileRepositoryPort;
use domain::{HealthGoal, UserId, UserProfile};

use super::models::UserProfileRow;

#[derive(Clone)]
pub struct PsqlUserRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl PsqlUserRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    pub async fn upsert_profile(&self, profile: &UserProfile) -> Result<(), String> {
        let id = profile.id.as_str().to_string();
        let mut db = self.db.lock().await;
        match UserProfileRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .display_name(profile.display_name.clone())
                    .introduction(profile.introduction.clone())
                    .health_goal(profile.health_goal.as_str().to_string())
                    .allergies_json(
                        serde_json::to_string(&profile.allergies).map_err(|err| err.to_string())?,
                    )
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
                Ok(())
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(UserProfileRow {
                    id: id,
                    display_name: profile.display_name.clone(),
                    introduction: profile.introduction.clone(),
                    health_goal: profile.health_goal.as_str().to_string(),
                    allergies_json: serde_json::to_string(&profile.allergies)
                        .map_err(|serialize_err| serialize_err.to_string())?,
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn row_to_profile(row: UserProfileRow) -> Result<UserProfile, String> {
        let allergies = serde_json::from_str::<Vec<String>>(&row.allergies_json)
            .map_err(|err| err.to_string())?;
        Ok(UserProfile {
            id: UserId::parse(&row.id).map_err(|err| err.to_string())?,
            display_name: row.display_name,
            introduction: row.introduction,
            health_goal: HealthGoal::new(row.health_goal),
            allergies,
        })
    }
}

#[async_trait]
impl UserProfileRepositoryPort for PsqlUserRepo {
    async fn find_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>, String> {
        let mut db = self.db.lock().await;
        let id = user_id.as_str().to_string();
        match UserProfileRow::get_by_id(&mut *db, &id).await {
            Ok(row) => Self::row_to_profile(row).map(Some),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}
