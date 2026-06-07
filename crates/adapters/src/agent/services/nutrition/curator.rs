use app::app_error::{AppError, AppResult};
use app::user::ResolvedAIModelConfig;
use async_trait::async_trait;
use domain::{FoodNutritionReference, FoodNutritionReferenceStatus, Nutrition};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{deepseek, openai};
use serde::Deserialize;

use crate::agent::prompts::nutrition_curator::NUTRITION_CURATOR_PREAMBLE;

#[async_trait]
pub trait NutritionCurator: Send + Sync {
    async fn curate(
        &self,
        food_name: &str,
        estimated_grams: Option<f32>,
    ) -> AppResult<Option<CuratedNutritionReference>>;
}

#[derive(Debug, Clone)]
pub struct CuratedNutritionReference {
    pub reference: FoodNutritionReference,
    pub confidence: f32,
    pub source: String,
}

#[derive(Clone)]
pub struct ModelNutritionCurator {
    config: ResolvedAIModelConfig,
}

impl ModelNutritionCurator {
    pub fn new(config: ResolvedAIModelConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NutritionCurator for ModelNutritionCurator {
    async fn curate(
        &self,
        food_name: &str,
        estimated_grams: Option<f32>,
    ) -> AppResult<Option<CuratedNutritionReference>> {
        let prompt = build_curator_prompt(food_name, estimated_grams);
        let text = match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|err| AppError::upstream(err.to_string()))?;
                client
                    .agent(self.config.model.as_str())
                    .preamble(NUTRITION_CURATOR_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|err| AppError::upstream(err.to_string()))?
            }
            "openai_compatible" | "siliconflow" | "dashscope" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|err| AppError::upstream(err.to_string()))?
                    .completions_api();
                client
                    .agent(self.config.model.as_str())
                    .preamble(NUTRITION_CURATOR_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|err| AppError::upstream(err.to_string()))?
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|err| AppError::upstream(err.to_string()))?;
                client
                    .agent(self.config.model.as_str())
                    .preamble(NUTRITION_CURATOR_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|err| AppError::upstream(err.to_string()))?
            }
            provider => {
                return Err(AppError::internal(format!(
                    "unsupported nutrition curator provider: {provider}"
                )));
            }
        };

        parse_curated_reference(&text)
    }
}

#[derive(Debug, Deserialize)]
struct CuratedReferenceDTO {
    id: String,
    name: String,
    #[serde(default)]
    terms: Vec<String>,
    #[serde(default = "default_basis_quantity")]
    basis_quantity: f32,
    #[serde(default = "default_basis_unit")]
    basis_unit: String,
    nutrition: Nutrition,
    #[serde(default)]
    confidence: f32,
    #[serde(default)]
    source: String,
}

fn build_curator_prompt(food_name: &str, estimated_grams: Option<f32>) -> String {
    let amount_context = estimated_grams
        .filter(|grams| *grams > 0.0)
        .map(|grams| format!("本次摄入估算重量约 {grams:.0} g。请用这个重量理解食物形态，但输出仍必须是每 100g 可食部分营养。"))
        .unwrap_or_else(|| "本次摄入重量未知。输出仍必须是每 100g 可食部分营养。".to_string());
    format!(
        r#"为食物补全一个本地营养参考：{food_name}
{amount_context}

只输出 JSON object，不要 Markdown，不要解释。
要求：
- basis 必须是 100 g 可食部分
- name 是业务 canonical display text，只保留一个最自然的名称
- terms 放常见说法、同义词、英文名或地区叫法，用于语义检索；不要过度合并不同食材
- nutrition 必须给出每 100g 的 calories, protein_g, fat_g, carbs_g；能确定时再填其他微量营养
- calories 必须和三大营养大致自洽：protein_g*4 + carbs_g*4 + fat_g*9 应接近 calories
- 即使缺少权威来源，也要给出可用的模型估算，不要输出 0 营养
- confidence 取 0 到 1；不确定时可以较低，但不要因为不确定而拒绝估算
- source 写明依据类型，例如 model_estimate、USDA/FDC estimate、CN food table estimate

JSON schema:
{{
  "id": "snake_case_canonical_id",
  "name": "canonical display name",
  "terms": ["common term", "synonym", "regional name"],
  "basis_quantity": 100.0,
  "basis_unit": "g",
  "nutrition": {{
    "calories": 0.0,
    "protein_g": 0.0,
    "fat_g": 0.0,
    "carbs_g": 0.0
  }},
  "confidence": 0.0,
  "source": "model_estimate"
}}"#
    )
}

fn parse_curated_reference(text: &str) -> AppResult<Option<CuratedNutritionReference>> {
    let Some(json) = extract_json_object(text) else {
        return Ok(None);
    };
    let dto = serde_json::from_str::<CuratedReferenceDTO>(json)
        .map_err(|err| AppError::upstream(format!("parse curated nutrition reference: {err}")))?;

    if !looks_plausible(&dto) {
        return Ok(None);
    }
    let confidence = dto.confidence.clamp(0.0, 1.0);
    let source = if dto.source.trim().is_empty() {
        "model_estimate".to_string()
    } else {
        dto.source
    };
    let nutrition = normalize_nutrition(dto.nutrition);

    Ok(Some(CuratedNutritionReference {
        reference: FoodNutritionReference {
            id: normalize_reference_id(&dto.id),
            name: dto.name,
            terms: dto.terms,
            basis_quantity: dto.basis_quantity,
            basis_unit: dto.basis_unit,
            nutrition,
            status: FoodNutritionReferenceStatus::Curated,
            source: Some(source.clone()),
            confidence: Some(confidence),
        },
        confidence,
        source,
    }))
}

fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    (start <= end).then_some(&text[start..=end])
}

fn looks_plausible(dto: &CuratedReferenceDTO) -> bool {
    let unit = dto.basis_unit.trim().to_lowercase();
    let nutrition = &dto.nutrition;
    let macro_calories = nutrition.protein_g.max(0.0) * 4.0
        + nutrition.carbs_g.max(0.0) * 4.0
        + nutrition.fat_g.max(0.0) * 9.0;
    !dto.id.trim().is_empty()
        && !dto.name.trim().is_empty()
        && (dto.basis_quantity - 100.0).abs() <= 0.01
        && matches!(unit.as_str(), "g" | "克")
        && nutrition.calories >= 0.0
        && nutrition.calories <= 900.0
        && nutrition.protein_g >= 0.0
        && nutrition.protein_g <= 100.0
        && nutrition.fat_g >= 0.0
        && nutrition.fat_g <= 100.0
        && nutrition.carbs_g >= 0.0
        && nutrition.carbs_g <= 100.0
        && nutrition.protein_g + nutrition.fat_g + nutrition.carbs_g <= 100.0
        && macro_calories > 0.0
}

fn normalize_nutrition(mut nutrition: Nutrition) -> Nutrition {
    let macro_calories = nutrition.protein_g.max(0.0) * 4.0
        + nutrition.carbs_g.max(0.0) * 4.0
        + nutrition.fat_g.max(0.0) * 9.0;
    if macro_calories > 0.0 {
        let delta = (nutrition.calories - macro_calories).abs();
        let tolerance = nutrition.calories.max(120.0) * 0.35;
        if nutrition.calories <= 0.0 || delta > tolerance {
            nutrition.calories = macro_calories;
        }
    }
    nutrition
}

fn normalize_reference_id(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn default_basis_quantity() -> f32 {
    100.0
}

fn default_basis_unit() -> String {
    "g".to_string()
}
