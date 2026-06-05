use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::meal::{
    estimate_nutrition_from_foods_with_references, FoodNutritionReferenceRepositoryPort,
};
use domain::{FoodItem, FoodNutritionReference, Nutrition};

use crate::agent::chains::meal_intake::{critique_foods, MealIntakeCritique};
use crate::agent::services::nutrition::curator::NutritionCurator;
use crate::agent::services::nutrition::retriever::NutritionReferenceRetriever;
use crate::agent::services::AgentServiceProgress;

#[derive(Debug, Clone)]
pub struct MealNutritionEstimate {
    pub nutrition: Nutrition,
    pub foods: Vec<ResolvedMealFood>,
    pub gaps: Vec<NutritionKnowledgeGap>,
    pub curated_references: Vec<CuratedNutritionReferenceRecord>,
    pub critiques: Vec<MealIntakeCritique>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMealFood {
    pub item: FoodItem,
    pub reference: Option<FoodNutritionReference>,
}

#[derive(Debug, Clone)]
pub struct NutritionKnowledgeGap {
    pub food_name: String,
    pub reason: NutritionKnowledgeGapReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NutritionKnowledgeGapReason {
    MissingReference,
}

#[derive(Debug, Clone)]
pub struct CuratedNutritionReferenceRecord {
    pub food_name: String,
    pub reference_id: String,
    pub source: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealNutritionStage {
    ReferenceResolved,
    ReferenceMissing,
    ReferenceCurated,
    Estimated,
    Critiqued,
}

#[derive(Clone, Default)]
pub struct MealNutritionEstimator {
    reference_repo: Option<Arc<dyn FoodNutritionReferenceRepositoryPort>>,
    curator: Option<Arc<dyn NutritionCurator>>,
    retriever: Option<Arc<dyn NutritionReferenceRetriever>>,
    progress: Option<Arc<dyn AgentServiceProgress>>,
}

impl MealNutritionEstimator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_reference_repo(
        mut self,
        reference_repo: Arc<dyn FoodNutritionReferenceRepositoryPort>,
    ) -> Self {
        self.reference_repo = Some(reference_repo);
        self
    }

    pub fn with_curator(mut self, curator: Arc<dyn NutritionCurator>) -> Self {
        self.curator = Some(curator);
        self
    }

    pub fn with_retriever(mut self, retriever: Arc<dyn NutritionReferenceRetriever>) -> Self {
        self.retriever = Some(retriever);
        self
    }

    pub fn with_progress(mut self, progress: Arc<dyn AgentServiceProgress>) -> Self {
        self.progress = Some(progress);
        self
    }

    pub async fn estimate(&self, foods: &[FoodItem]) -> AppResult<MealNutritionEstimate> {
        if foods.is_empty() {
            return Err(AppError::validation("meal contains no food items"));
        }

        let mut resolved_foods = self.resolve_references(foods).await;
        let curated_references = self.curate_missing_references(&mut resolved_foods).await;
        let gaps = unresolved_gaps(&resolved_foods);
        tracing::info!(
            stage = ?MealNutritionStage::ReferenceResolved,
            food.count = resolved_foods.len(),
            gap.count = gaps.len(),
            "meal nutrition estimation stage completed"
        );
        if !gaps.is_empty() {
            tracing::info!(
                stage = ?MealNutritionStage::ReferenceMissing,
                foods = ?gaps.iter().map(|gap| gap.food_name.as_str()).collect::<Vec<_>>(),
                "meal nutrition estimation found reference gaps"
            );
        }

        let references = resolved_foods
            .iter()
            .map(|food| food.reference.clone())
            .collect::<Vec<_>>();
        let nutrition = estimate_nutrition_from_foods_with_references(foods, &references);
        tracing::info!(
            stage = ?MealNutritionStage::Estimated,
            calories = nutrition.calories,
            "meal nutrition estimation stage completed"
        );

        let critiques = critique_foods(foods);
        tracing::info!(
            stage = ?MealNutritionStage::Critiqued,
            critique.count = critiques.len(),
            "meal nutrition estimation stage completed"
        );

        Ok(MealNutritionEstimate {
            nutrition,
            foods: resolved_foods,
            gaps,
            curated_references,
            critiques,
        })
    }

    async fn resolve_references(&self, foods: &[FoodItem]) -> Vec<ResolvedMealFood> {
        let Some(retriever) = &self.retriever else {
            return foods
                .iter()
                .cloned()
                .map(|item| ResolvedMealFood {
                    item,
                    reference: None,
                })
                .collect();
        };

        let mut resolved = Vec::with_capacity(foods.len());
        for item in foods {
            let reference = match retriever.retrieve(&item.name).await {
                Ok(reference) => reference,
                Err(err) => {
                    tracing::warn!(
                        food.name = %item.name,
                        error = %err,
                        "failed to retrieve nutrition reference"
                    );
                    None
                }
            };

            resolved.push(ResolvedMealFood {
                item: item.clone(),
                reference,
            });
        }

        resolved
    }

    async fn curate_missing_references(
        &self,
        foods: &mut [ResolvedMealFood],
    ) -> Vec<CuratedNutritionReferenceRecord> {
        let Some(curator) = &self.curator else {
            return Vec::new();
        };
        let Some(reference_repo) = &self.reference_repo else {
            return Vec::new();
        };

        let mut records = Vec::new();
        for food in foods.iter_mut().filter(|food| food.reference.is_none()) {
            self.emit_status("我在补充这份食物的营养参考。");
            match curator
                .curate(&food.item.name, food.item.estimated_grams)
                .await
            {
                Ok(Some(curated)) => {
                    let reference_id = curated.reference.id.clone();
                    match reference_repo.upsert_reference(&curated.reference).await {
                        Ok(()) => {
                            if let Some(retriever) = &self.retriever {
                                if let Err(err) = retriever.put(&curated.reference).await {
                                    tracing::warn!(
                                        stage = ?MealNutritionStage::ReferenceCurated,
                                        food.name = %food.item.name,
                                        error = %err,
                                        "failed to index curated nutrition reference"
                                    );
                                }
                            }
                            tracing::info!(
                                stage = ?MealNutritionStage::ReferenceCurated,
                                food.name = %food.item.name,
                                reference.id = %reference_id,
                                confidence = curated.confidence,
                                source = %curated.source,
                                "nutrition reference curated"
                            );
                            food.reference = Some(curated.reference);
                            records.push(CuratedNutritionReferenceRecord {
                                food_name: food.item.name.clone(),
                                reference_id,
                                source: curated.source,
                                confidence: curated.confidence,
                            });
                        }
                        Err(err) => {
                            tracing::warn!(
                                stage = ?MealNutritionStage::ReferenceCurated,
                                food.name = %food.item.name,
                                error = %err,
                                "failed to persist curated nutrition reference"
                            );
                        }
                    }
                }
                Ok(None) => {
                    tracing::info!(
                        stage = ?MealNutritionStage::ReferenceCurated,
                        food.name = %food.item.name,
                        "nutrition curator returned no acceptable reference"
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        stage = ?MealNutritionStage::ReferenceCurated,
                        food.name = %food.item.name,
                        error = %err,
                        "nutrition curator failed"
                    );
                }
            }
        }

        if !records.is_empty() {
            tracing::info!(
                stage = ?MealNutritionStage::ReferenceCurated,
                curated.count = records.len(),
                "meal nutrition estimation stage completed"
            );
        }

        records
    }

    fn emit_status(&self, label: &'static str) {
        if let Some(progress) = &self.progress {
            progress.status(label);
        }
    }
}

fn unresolved_gaps(foods: &[ResolvedMealFood]) -> Vec<NutritionKnowledgeGap> {
    foods
        .iter()
        .filter_map(|food| {
            food.reference.is_none().then(|| NutritionKnowledgeGap {
                food_name: food.item.name.clone(),
                reason: NutritionKnowledgeGapReason::MissingReference,
            })
        })
        .collect()
}
