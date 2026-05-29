use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use app::meal::{
    FoodNutritionReferenceRepositoryPort, MealDayFinalizationRepositoryPort,
    MealDaySummaryRepositoryPort, MealRecordRepositoryPort, PendingMealLogRepositoryPort,
};
use domain::{
    DayCycle, FoodNutritionReference, MealDayFinalization, MealDaySummary, MealRecord,
    PendingMealLog, PendingMealLogId, PendingMealLogStatus, UserId,
};

use crate::persistence::sqlite::models::{
    FoodNutritionReferenceRow, MealDayFinalizationRow, MealDaySummaryRow, MealRecordRow,
    PendingMealLogRow,
};

#[derive(Clone)]
pub struct SqliteMealRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteMealRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    pub async fn is_ready(&self) -> bool {
        let _db_guard = self.db.lock().await;
        true
    }

    async fn upsert_reference_id(
        &self,
        lookup_key: String,
        reference: &FoodNutritionReference,
    ) -> Result<(), String> {
        let labels_json =
            serde_json::to_string(&reference.labels).map_err(|err| err.to_string())?;
        let aliases_json =
            serde_json::to_string(&reference.aliases).map_err(|err| err.to_string())?;
        let nutrition_json =
            serde_json::to_string(&reference.nutrition).map_err(|err| err.to_string())?;
        let mut db = self.db.lock().await;

        match FoodNutritionReferenceRow::get_by_id(&mut *db, &lookup_key).await {
            Ok(mut current) => {
                current
                    .update()
                    .reference_id(reference.id.clone())
                    .labels_json(labels_json)
                    .aliases_json(aliases_json)
                    .basis_quantity(reference.basis_quantity)
                    .basis_unit(reference.basis_unit.clone())
                    .nutrition_json(nutrition_json)
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(FoodNutritionReferenceRow {
                    id: lookup_key,
                    reference_id: reference.id.clone(),
                    labels_json: labels_json,
                    aliases_json: aliases_json,
                    basis_quantity: reference.basis_quantity,
                    basis_unit: reference.basis_unit.clone(),
                    nutrition_json: nutrition_json,
                })
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }

        Ok(())
    }

    fn row_to_reference(row: FoodNutritionReferenceRow) -> Result<FoodNutritionReference, String> {
        Ok(FoodNutritionReference {
            id: row.reference_id,
            labels: serde_json::from_str(&row.labels_json).map_err(|err| err.to_string())?,
            aliases: serde_json::from_str(&row.aliases_json).map_err(|err| err.to_string())?,
            basis_quantity: row.basis_quantity,
            basis_unit: row.basis_unit,
            nutrition: serde_json::from_str(&row.nutrition_json).map_err(|err| err.to_string())?,
        })
    }

    fn row_to_pending(row: PendingMealLogRow) -> Result<PendingMealLog, String> {
        Ok(PendingMealLog {
            id: PendingMealLogId::parse(&row.id).map_err(|err| err.to_string())?,
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            session_id: row.session_id,
            day_cycle: DayCycle::parse(&row.day_cycle).map_err(|err| err.to_string())?,
            foods: serde_json::from_str(&row.foods_json).map_err(|err| err.to_string())?,
            nutrition: serde_json::from_str(&row.nutrition_json).map_err(|err| err.to_string())?,
            status: pending_status_from_str(&row.status)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    fn row_to_meal(row: MealRecordRow) -> Result<MealRecord, String> {
        Ok(MealRecord {
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            session_id: row.session_id,
            day_cycle: DayCycle::parse(&row.day_cycle).map_err(|err| err.to_string())?,
            foods: serde_json::from_str(&row.foods_json).map_err(|err| err.to_string())?,
            nutrition: serde_json::from_str(&row.nutrition_json).map_err(|err| err.to_string())?,
        })
    }

    fn row_to_summary(row: MealDaySummaryRow) -> Result<MealDaySummary, String> {
        Ok(MealDaySummary {
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            session_id: row.session_id,
            content: row.content,
            nutrition_score: row.nutrition_score,
            expectation_match_score: row.expectation_match_score,
            overall_score: row.overall_score,
            metrics_json: row.metrics_json,
            finalized_at: row.finalized_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl MealRecordRepositoryPort for SqliteMealRepo {
    async fn save_meal(&self, meal: &MealRecord) -> Result<(), String> {
        let foods_json = serde_json::to_string(&meal.foods).map_err(|err| err.to_string())?;
        let nutrition_json =
            serde_json::to_string(&meal.nutrition).map_err(|err| err.to_string())?;
        let mut db = self.db.lock().await;

        toasty::create!(MealRecordRow {
            user_id: meal.user_id.as_str().to_string(),
            session_id: meal.session_id.clone(),
            day_cycle: meal.day_cycle.as_str().to_string(),
            foods_json: foods_json,
            nutrition_json: nutrition_json,
        })
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    async fn list_meals(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<MealRecord>, String> {
        let mut db = self.db.lock().await;
        let rows = MealRecordRow::filter(
            MealRecordRow::fields()
                .user_id()
                .eq(user_id.as_str())
                .and(MealRecordRow::fields().session_id().eq(session_id)),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        rows.into_iter().map(Self::row_to_meal).collect()
    }
}

#[async_trait]
impl MealDayFinalizationRepositoryPort for SqliteMealRepo {
    async fn save_finalization(&self, finalization: &MealDayFinalization) -> Result<(), String> {
        let id = meal_day_finalization_id(&finalization.user_id, &finalization.session_id);
        let mut db = self.db.lock().await;

        match MealDayFinalizationRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .user_id(finalization.user_id.as_str().to_string())
                    .session_id(finalization.session_id.clone())
                    .finalized_at(finalization.finalized_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(MealDayFinalizationRow {
                    id: id,
                    user_id: finalization.user_id.as_str().to_string(),
                    session_id: finalization.session_id.clone(),
                    finalized_at: finalization.finalized_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }

        Ok(())
    }

    async fn find_finalization(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Option<MealDayFinalization>, String> {
        let id = meal_day_finalization_id(user_id, session_id);
        let mut db = self.db.lock().await;
        match MealDayFinalizationRow::get_by_id(&mut *db, &id).await {
            Ok(row) if row.user_id == user_id.as_str() => Ok(Some(MealDayFinalization {
                user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
                session_id: row.session_id,
                finalized_at: row.finalized_at,
            })),
            Ok(_) => Ok(None),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}

#[async_trait]
impl MealDaySummaryRepositoryPort for SqliteMealRepo {
    async fn save_summary(&self, summary: &MealDaySummary) -> Result<(), String> {
        let id = meal_day_summary_id(&summary.user_id, &summary.session_id);
        let mut db = self.db.lock().await;

        match MealDaySummaryRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .user_id(summary.user_id.as_str().to_string())
                    .session_id(summary.session_id.clone())
                    .content(summary.content.clone())
                    .nutrition_score(summary.nutrition_score)
                    .expectation_match_score(summary.expectation_match_score)
                    .overall_score(summary.overall_score)
                    .metrics_json(summary.metrics_json.clone())
                    .finalized_at(summary.finalized_at.clone())
                    .created_at(summary.created_at.clone())
                    .updated_at(summary.updated_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(MealDaySummaryRow {
                    id: id,
                    user_id: summary.user_id.as_str().to_string(),
                    session_id: summary.session_id.clone(),
                    content: summary.content.clone(),
                    nutrition_score: summary.nutrition_score,
                    expectation_match_score: summary.expectation_match_score,
                    overall_score: summary.overall_score,
                    metrics_json: summary.metrics_json.clone(),
                    finalized_at: summary.finalized_at.clone(),
                    created_at: summary.created_at.clone(),
                    updated_at: summary.updated_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }

        Ok(())
    }

    async fn find_summary(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Option<MealDaySummary>, String> {
        let id = meal_day_summary_id(user_id, session_id);
        let mut db = self.db.lock().await;
        match MealDaySummaryRow::get_by_id(&mut *db, &id).await {
            Ok(row) if row.user_id == user_id.as_str() => Self::row_to_summary(row).map(Some),
            Ok(_) => Ok(None),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn list_summaries(&self, user_id: &UserId) -> Result<Vec<MealDaySummary>, String> {
        let mut db = self.db.lock().await;
        let rows = MealDaySummaryRow::filter(
            MealDaySummaryRow::fields()
                .user_id()
                .eq(user_id.as_str()),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        let mut summaries = rows
            .into_iter()
            .map(Self::row_to_summary)
            .collect::<Result<Vec<_>, _>>()?;
        summaries.sort_by(|left, right| right.finalized_at.cmp(&left.finalized_at));
        Ok(summaries)
    }
}

#[async_trait]
impl PendingMealLogRepositoryPort for SqliteMealRepo {
    async fn save_pending_meal(&self, pending: &PendingMealLog) -> Result<(), String> {
        let id = pending.id.as_str().to_string();
        let foods_json = serde_json::to_string(&pending.foods).map_err(|err| err.to_string())?;
        let nutrition_json =
            serde_json::to_string(&pending.nutrition).map_err(|err| err.to_string())?;
        let status = pending_status_to_str(&pending.status).to_string();
        let mut db = self.db.lock().await;

        match PendingMealLogRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .user_id(pending.user_id.as_str().to_string())
                    .session_id(pending.session_id.clone())
                    .day_cycle(pending.day_cycle.as_str().to_string())
                    .foods_json(foods_json)
                    .nutrition_json(nutrition_json)
                    .status(status)
                    .created_at(pending.created_at.clone())
                    .updated_at(pending.updated_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(PendingMealLogRow {
                    id: id,
                    user_id: pending.user_id.as_str().to_string(),
                    session_id: pending.session_id.clone(),
                    day_cycle: pending.day_cycle.as_str().to_string(),
                    foods_json: foods_json,
                    nutrition_json: nutrition_json,
                    status: status,
                    created_at: pending.created_at.clone(),
                    updated_at: pending.updated_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }
        Ok(())
    }

    async fn find_pending_meal(
        &self,
        user_id: &UserId,
        id: &PendingMealLogId,
    ) -> Result<Option<PendingMealLog>, String> {
        let mut db = self.db.lock().await;
        match PendingMealLogRow::get_by_id(&mut *db, id.as_str()).await {
            Ok(row) if row.user_id == user_id.as_str() => Self::row_to_pending(row).map(Some),
            Ok(_) => Ok(None),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn list_pending_meals(
        &self,
        user_id: &UserId,
        session_id: &str,
    ) -> Result<Vec<PendingMealLog>, String> {
        let mut db = self.db.lock().await;
        let rows = PendingMealLogRow::filter(
            PendingMealLogRow::fields()
                .user_id()
                .eq(user_id.as_str())
                .and(PendingMealLogRow::fields().session_id().eq(session_id)),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        let mut items = rows
            .into_iter()
            .map(Self::row_to_pending)
            .collect::<Result<Vec<_>, _>>()?;
        items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(items)
    }

    async fn delete_pending_meal(
        &self,
        user_id: &UserId,
        id: &PendingMealLogId,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let row = PendingMealLogRow::get_by_id(&mut *db, id.as_str())
            .await
            .map_err(|err| err.to_string())?;

        if row.user_id != user_id.as_str() {
            return Err("pending meal log does not belong to user".to_string());
        }

        row.delete()
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())
    }
}

#[async_trait]
impl FoodNutritionReferenceRepositoryPort for SqliteMealRepo {
    async fn upsert_reference(&self, reference: &FoodNutritionReference) -> Result<(), String> {
        let mut ids = reference
            .search_terms()
            .iter()
            .map(|term| normalize_food_name(term))
            .collect::<Vec<_>>();
        ids.push(normalize_food_name(&reference.id));
        ids.retain(|id| !id.is_empty());
        ids.sort();
        ids.dedup();

        for id in ids {
            self.upsert_reference_id(id, reference).await?;
        }

        Ok(())
    }

    async fn find_reference_by_name(
        &self,
        name: &str,
    ) -> Result<Option<FoodNutritionReference>, String> {
        let id = normalize_food_name(name);
        if id.is_empty() {
            return Ok(None);
        }

        let mut db = self.db.lock().await;
        match FoodNutritionReferenceRow::get_by_id(&mut *db, &id).await {
            Ok(row) => Self::row_to_reference(row).map(Some),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}

fn normalize_food_name(value: &str) -> String {
    value.trim().to_lowercase().replace(' ', "")
}

fn pending_status_to_str(status: &PendingMealLogStatus) -> &'static str {
    match status {
        PendingMealLogStatus::Proposed => "proposed",
        PendingMealLogStatus::Confirmed => "confirmed",
        PendingMealLogStatus::Rejected => "rejected",
    }
}

fn pending_status_from_str(value: &str) -> Result<PendingMealLogStatus, String> {
    match value {
        "proposed" => Ok(PendingMealLogStatus::Proposed),
        "confirmed" => Ok(PendingMealLogStatus::Confirmed),
        "rejected" => Ok(PendingMealLogStatus::Rejected),
        other => Err(format!("unknown pending meal status: {other}")),
    }
}

fn meal_day_finalization_id(user_id: &UserId, session_id: &str) -> String {
    format!("{}:{}", user_id.as_str(), session_id)
}

fn meal_day_summary_id(user_id: &UserId, session_id: &str) -> String {
    format!("{}:{}", user_id.as_str(), session_id)
}
