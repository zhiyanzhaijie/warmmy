use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayCycle(String);

impl DayCycle {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, DayCycleInvalidError> {
        let value = value.trim().to_lowercase();
        match value.as_str() {
            "breakfast" | "lunch" | "dinner" | "snack" => Ok(Self(value)),
            _ => Err(DayCycleInvalidError { value }),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DayCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayCycleInvalidError {
    value: String,
}

impl std::fmt::Display for DayCycleInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "meal: invalid day cycle: '{}', expected one of breakfast|lunch|dinner|snack",
            self.value
        )
    }
}

impl std::error::Error for DayCycleInvalidError {}

impl crate::error::DomainError for DayCycleInvalidError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PendingMealLogId(String);

impl PendingMealLogId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, DayCycleInvalidError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(DayCycleInvalidError {
                value: value.to_string(),
            });
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PendingMealLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingMealLogStatus {
    Proposed,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Nutrition {
    #[serde(default)]
    pub calories: f32,
    #[serde(default)]
    pub protein_g: f32,
    #[serde(default)]
    pub fat_g: f32,
    #[serde(default)]
    pub carbs_g: f32,
    #[serde(default)]
    pub fiber_g: f32,
    #[serde(default)]
    pub sugar_g: f32,
    #[serde(default)]
    pub saturated_fat_g: f32,
    #[serde(default)]
    pub sodium_mg: f32,
    #[serde(default)]
    pub potassium_mg: f32,
    #[serde(default)]
    pub calcium_mg: f32,
    #[serde(default)]
    pub iron_mg: f32,
    #[serde(default)]
    pub magnesium_mg: f32,
    #[serde(default)]
    pub zinc_mg: f32,
    #[serde(default)]
    pub vitamin_a_rae_ug: f32,
    #[serde(default)]
    pub vitamin_c_mg: f32,
    #[serde(default)]
    pub vitamin_d_ug: f32,
    #[serde(default)]
    pub vitamin_e_mg: f32,
    #[serde(default)]
    pub vitamin_k_ug: f32,
    #[serde(default)]
    pub thiamin_b1_mg: f32,
    #[serde(default)]
    pub riboflavin_b2_mg: f32,
    #[serde(default)]
    pub niacin_b3_mg: f32,
    #[serde(default)]
    pub vitamin_b6_mg: f32,
    #[serde(default)]
    pub folate_ug: f32,
    #[serde(default)]
    pub vitamin_b12_ug: f32,
}

impl Nutrition {
    pub fn add(&self, other: &Nutrition) -> Nutrition {
        Nutrition {
            calories: self.calories + other.calories,
            protein_g: self.protein_g + other.protein_g,
            fat_g: self.fat_g + other.fat_g,
            carbs_g: self.carbs_g + other.carbs_g,
            fiber_g: self.fiber_g + other.fiber_g,
            sugar_g: self.sugar_g + other.sugar_g,
            saturated_fat_g: self.saturated_fat_g + other.saturated_fat_g,
            sodium_mg: self.sodium_mg + other.sodium_mg,
            potassium_mg: self.potassium_mg + other.potassium_mg,
            calcium_mg: self.calcium_mg + other.calcium_mg,
            iron_mg: self.iron_mg + other.iron_mg,
            magnesium_mg: self.magnesium_mg + other.magnesium_mg,
            zinc_mg: self.zinc_mg + other.zinc_mg,
            vitamin_a_rae_ug: self.vitamin_a_rae_ug + other.vitamin_a_rae_ug,
            vitamin_c_mg: self.vitamin_c_mg + other.vitamin_c_mg,
            vitamin_d_ug: self.vitamin_d_ug + other.vitamin_d_ug,
            vitamin_e_mg: self.vitamin_e_mg + other.vitamin_e_mg,
            vitamin_k_ug: self.vitamin_k_ug + other.vitamin_k_ug,
            thiamin_b1_mg: self.thiamin_b1_mg + other.thiamin_b1_mg,
            riboflavin_b2_mg: self.riboflavin_b2_mg + other.riboflavin_b2_mg,
            niacin_b3_mg: self.niacin_b3_mg + other.niacin_b3_mg,
            vitamin_b6_mg: self.vitamin_b6_mg + other.vitamin_b6_mg,
            folate_ug: self.folate_ug + other.folate_ug,
            vitamin_b12_ug: self.vitamin_b12_ug + other.vitamin_b12_ug,
        }
    }

    pub fn scale(&self, factor: f32) -> Nutrition {
        Nutrition {
            calories: self.calories * factor,
            protein_g: self.protein_g * factor,
            fat_g: self.fat_g * factor,
            carbs_g: self.carbs_g * factor,
            fiber_g: self.fiber_g * factor,
            sugar_g: self.sugar_g * factor,
            saturated_fat_g: self.saturated_fat_g * factor,
            sodium_mg: self.sodium_mg * factor,
            potassium_mg: self.potassium_mg * factor,
            calcium_mg: self.calcium_mg * factor,
            iron_mg: self.iron_mg * factor,
            magnesium_mg: self.magnesium_mg * factor,
            zinc_mg: self.zinc_mg * factor,
            vitamin_a_rae_ug: self.vitamin_a_rae_ug * factor,
            vitamin_c_mg: self.vitamin_c_mg * factor,
            vitamin_d_ug: self.vitamin_d_ug * factor,
            vitamin_e_mg: self.vitamin_e_mg * factor,
            vitamin_k_ug: self.vitamin_k_ug * factor,
            thiamin_b1_mg: self.thiamin_b1_mg * factor,
            riboflavin_b2_mg: self.riboflavin_b2_mg * factor,
            niacin_b3_mg: self.niacin_b3_mg * factor,
            vitamin_b6_mg: self.vitamin_b6_mg * factor,
            folate_ug: self.folate_ug * factor,
            vitamin_b12_ug: self.vitamin_b12_ug * factor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FoodNutritionReference {
    pub id: String,
    pub labels: BTreeMap<String, String>,
    pub aliases: BTreeMap<String, Vec<String>>,
    pub basis_quantity: f32,
    pub basis_unit: String,
    pub nutrition: Nutrition,
}

impl FoodNutritionReference {
    pub fn search_terms(&self) -> Vec<String> {
        let mut terms = self.labels.values().cloned().collect::<Vec<_>>();
        for aliases in self.aliases.values() {
            terms.extend(aliases.iter().cloned());
        }
        terms
    }
}
