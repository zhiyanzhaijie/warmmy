use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use app::meal::{FoodNutritionReferenceRepositoryPort, MealRecordRepositoryPort};
use domain::{FoodNutritionReference, MealRecord};

use crate::persistence::sqlite::models::{FoodNutritionReferenceRow, MealRecordRow};

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
            day_cycle: meal.day_cycle.as_str().to_string(),
            foods_json: foods_json,
            nutrition_json: nutrition_json,
        })
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
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
