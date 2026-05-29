use api::meal::MealDaySummaryDTO;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SummaryMetrics {
    #[serde(default)]
    pub meal_count: usize,
    #[serde(default)]
    pub total_nutrition: NutritionMetrics,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct NutritionMetrics {
    #[serde(default)]
    pub calories: f32,
    #[serde(default)]
    pub protein_g: f32,
    #[serde(default)]
    pub fat_g: f32,
    #[serde(default)]
    pub carbs_g: f32,
}

pub fn parse_metrics(summary: &MealDaySummaryDTO) -> SummaryMetrics {
    serde_json::from_str(&summary.metrics_json).unwrap_or_default()
}

pub fn date_label(session_id: &str) -> String {
    chrono::NaiveDate::parse_from_str(session_id, "%Y-%m-%d")
        .map(|date| date.format("%m.%d").to_string())
        .unwrap_or_else(|_| session_id.to_string())
}

pub fn detail_date_label(session_id: &str) -> String {
    chrono::NaiveDate::parse_from_str(session_id, "%Y-%m-%d")
        .map(|date| date.format("%Y.%m.%d").to_string())
        .unwrap_or_else(|_| session_id.to_string())
}

pub fn compact_number(value: f32) -> String {
    if value.abs() >= 100.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}
