use crate::impls::error::api_error;
use dioxus::fullstack::payloads::TextStream;
use dioxus::prelude::*;
use futures_util::StreamExt;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct FoodItemDTO {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct NutritionDTO {
    pub calories: f32,
    pub protein_g: f32,
    pub fat_g: f32,
    pub carbs_g: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MealRecordDTO {
    pub session_id: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemDTO>,
    pub nutrition: NutritionDTO,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct PendingMealLogDTO {
    pub id: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemDTO>,
    pub nutrition: NutritionDTO,
    pub status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ConfirmPendingMealInput {
    pub pending_id: String,
    pub day_cycle: String,
    pub foods: Vec<FoodItemDTO>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MealDayFinalizationDTO {
    pub session_id: String,
    pub finalized_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MealDaySummaryDTO {
    pub session_id: String,
    pub content: String,
    pub nutrition_score: f32,
    pub expectation_match_score: f32,
    pub overall_score: f32,
    pub metrics_json: String,
    pub finalized_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_meal_day_finalization(
    user_id: String,
    session_id: String,
) -> Result<Option<MealDayFinalizationDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let item = state
        .0
        .meal
        .command
        .find_meal_day_finalization(&user_id, &session_id)
        .await
        .map_err(api_error)?;
    Ok(item.map(finalization_to_dto))
}

pub async fn get_meal_day_summary(
    user_id: String,
    session_id: String,
) -> Result<Option<MealDaySummaryDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let item = state
        .0
        .meal
        .command
        .find_meal_day_summary(&user_id, &session_id)
        .await
        .map_err(api_error)?;
    Ok(item.map(summary_to_dto))
}

pub async fn list_meal_day_summaries(
    user_id: String,
) -> Result<Vec<MealDaySummaryDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let items = state
        .0
        .meal
        .command
        .list_meal_day_summaries(&user_id)
        .await
        .map_err(api_error)?;
    Ok(items.into_iter().map(summary_to_dto).collect())
}

pub async fn list_meal_logs(
    user_id: String,
    session_id: String,
) -> Result<Vec<MealRecordDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let items = state
        .0
        .meal
        .query
        .list_meals(&user_id, &session_id)
        .await
        .map_err(api_error)?;
    Ok(items.into_iter().map(meal_to_dto).collect())
}

pub async fn finalize_meal_day(
    user_id: String,
    session_id: String,
) -> Result<MealDayFinalizationDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let result = state
        .0
        .meal
        .command
        .finalize_meal_day(app::meal::FinalizeMealDayCommand {
            user_id: parse_user_id(&user_id)?,
            session_id,
        })
        .await
        .map_err(api_error)?;
    Ok(finalization_to_dto(result.finalization))
}

pub async fn finalize_and_summarize_meal_day(
    user_id: String,
    session_id: String,
) -> Result<TextStream, ServerFnError> {
    let state = crate::local_state::state().await?;
    let parsed_user_id = parse_user_id(&user_id)?;
    let result = state
        .0
        .meal
        .command
        .finalize_meal_day(app::meal::FinalizeMealDayCommand {
            user_id: parsed_user_id.clone(),
            session_id: session_id.clone(),
        })
        .await
        .map_err(api_error)?;

    let meals = state
        .0
        .meal
        .query
        .list_meals(&parsed_user_id, &session_id)
        .await
        .map_err(api_error)?;
    let meals_json = serde_json::to_string(&meals)
        .map_err(|err| ServerFnError::new(format!("serialize meals failed: {err}")))?;
    let finalized_at = result.finalization.finalized_at.clone();

    let stream = state
        .0
        .conversation
        .command
        .continue_interaction(app::conversation::ContinueInteractionCommand {
            user_id: parsed_user_id.clone(),
            session_id: session_id.clone(),
            interaction: app::conversation::AgentInteractionContinuation::SummarizeMealDay {
                finalized_at: finalized_at.clone(),
                meals_json,
            },
        })
        .await
        .map_err(api_error)?;

    let command = state.0.meal.command.clone();
    Ok(TextStream::new(async_stream::stream! {
        let mut content = String::new();
        let mut stream = stream;
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => {
                    if let Some(text) = decode_text_delta(&chunk) {
                        content.push_str(&text);
                    }
                    yield chunk;
                }
                Err(err) => {
                    yield format!("\n[stream error] {err}");
                    return;
                }
            }
        }

        if let Err(err) = command
            .save_meal_day_summary(app::meal::SaveMealDaySummaryCommand {
                user_id: parsed_user_id,
                session_id,
                content,
                finalized_at,
            })
            .await
        {
            yield format!("\n[summary save error] {err}");
        }
    }))
}

pub async fn list_pending_meals(
    user_id: String,
    session_id: String,
) -> Result<Vec<PendingMealLogDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let items = state
        .0
        .meal
        .command
        .list_pending_meals(&user_id, &session_id)
        .await
        .map_err(api_error)?;
    Ok(items.into_iter().map(pending_to_dto).collect())
}

pub async fn preview_pending_meal(
    user_id: String,
    input: ConfirmPendingMealInput,
) -> Result<PendingMealLogDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let pending = state
        .0
        .meal
        .command
        .update_pending_meal(app::meal::UpdatePendingMealLogCommand {
            user_id,
            pending_id: domain::PendingMealLogId::new_unchecked(input.pending_id),
            day_cycle: input.day_cycle,
            foods: input
                .foods
                .into_iter()
                .map(|food| domain::FoodItem::new(food.name, food.quantity, food.unit))
                .collect(),
        })
        .await
        .map_err(api_error)?;
    Ok(pending_to_dto(pending))
}

pub async fn confirm_pending_meal(
    user_id: String,
    session_id: String,
    input: ConfirmPendingMealInput,
) -> Result<TextStream, ServerFnError> {
    let state = crate::local_state::state().await?;
    let parsed_user_id = parse_user_id(&user_id)?;
    let pending = state
        .0
        .meal
        .command
        .update_pending_meal(app::meal::UpdatePendingMealLogCommand {
            user_id: parsed_user_id.clone(),
            pending_id: domain::PendingMealLogId::new_unchecked(input.pending_id),
            day_cycle: input.day_cycle,
            foods: input
                .foods
                .into_iter()
                .map(|food| domain::FoodItem::new(food.name, food.quantity, food.unit))
                .collect(),
        })
        .await
        .map_err(api_error)?;

    let stream = state
        .0
        .conversation
        .command
        .continue_interaction(app::conversation::ContinueInteractionCommand {
            user_id: parsed_user_id,
            session_id,
            interaction: app::conversation::AgentInteractionContinuation::ConfirmMealLog {
                pending_id: pending.id.to_string(),
            },
        })
        .await
        .map_err(api_error)?;
    Ok(TextStream::new(stream.map(|item| match item {
        Ok(chunk) => chunk,
        Err(err) => format!("\n[stream error] {err}"),
    })))
}

pub async fn reject_pending_meal(
    user_id: String,
    session_id: String,
    pending_id: String,
) -> Result<TextStream, ServerFnError> {
    let state = crate::local_state::state().await?;
    let stream = state
        .0
        .conversation
        .command
        .continue_interaction(app::conversation::ContinueInteractionCommand {
            user_id: parse_user_id(&user_id)?,
            session_id,
            interaction: app::conversation::AgentInteractionContinuation::RejectMealLog {
                pending_id,
            },
        })
        .await
        .map_err(api_error)?;
    Ok(TextStream::new(stream.map(|item| match item {
        Ok(chunk) => chunk,
        Err(err) => format!("\n[stream error] {err}"),
    })))
}

fn finalization_to_dto(item: domain::MealDayFinalization) -> MealDayFinalizationDTO {
    MealDayFinalizationDTO {
        session_id: item.session_id,
        finalized_at: item.finalized_at,
    }
}

fn summary_to_dto(item: domain::MealDaySummary) -> MealDaySummaryDTO {
    MealDaySummaryDTO {
        session_id: item.session_id,
        content: item.content,
        nutrition_score: item.nutrition_score,
        expectation_match_score: item.expectation_match_score,
        overall_score: item.overall_score,
        metrics_json: item.metrics_json,
        finalized_at: item.finalized_at,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn meal_to_dto(item: domain::MealRecord) -> MealRecordDTO {
    MealRecordDTO {
        session_id: item.session_id,
        day_cycle: item.day_cycle.to_string(),
        foods: item
            .foods
            .into_iter()
            .map(|food| FoodItemDTO {
                name: food.name,
                quantity: food.quantity,
                unit: food.unit,
            })
            .collect(),
        nutrition: nutrition_to_dto(item.nutrition),
    }
}

fn pending_to_dto(item: domain::PendingMealLog) -> PendingMealLogDTO {
    PendingMealLogDTO {
        id: item.id.to_string(),
        day_cycle: item.day_cycle.to_string(),
        foods: item
            .foods
            .into_iter()
            .map(|food| FoodItemDTO {
                name: food.name,
                quantity: food.quantity,
                unit: food.unit,
            })
            .collect(),
        nutrition: nutrition_to_dto(item.nutrition),
        status: match item.status {
            domain::PendingMealLogStatus::Proposed => "proposed",
            domain::PendingMealLogStatus::Confirmed => "confirmed",
            domain::PendingMealLogStatus::Rejected => "rejected",
        }
        .to_string(),
    }
}

fn nutrition_to_dto(nutrition: domain::Nutrition) -> NutritionDTO {
    NutritionDTO {
        calories: nutrition.calories,
        protein_g: nutrition.protein_g,
        fat_g: nutrition.fat_g,
        carbs_g: nutrition.carbs_g,
    }
}

fn parse_user_id(value: &str) -> Result<domain::UserId, ServerFnError> {
    domain::UserId::parse(value)
        .map_err(|err| ServerFnError::new(format!("invalid user_id `{}`: {}", value.trim(), err)))
}

fn decode_text_delta(chunk: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    #[serde(tag = "type")]
    enum Event {
        #[serde(rename = "text_delta")]
        TextDelta { text: String },
    }

    serde_json::from_str::<Event>(chunk.trim())
        .ok()
        .map(|event| match event {
            Event::TextDelta { text } => text,
        })
}
