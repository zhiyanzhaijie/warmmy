use std::sync::Arc;

use domain::{FoodItem, Nutrition};

use crate::meal::FoodNutritionReferenceRepositoryPort;

pub fn estimate_nutrition_from_foods(foods: &[FoodItem]) -> Nutrition {
    foods.iter().fold(Nutrition::default(), |acc, item| {
        let estimated = Nutrition {
            calories: 120.0 * item.quantity.max(0.0),
            protein_g: 4.0 * item.quantity.max(0.0),
            fat_g: 3.0 * item.quantity.max(0.0),
            carbs_g: 12.0 * item.quantity.max(0.0),
            fiber_g: 1.0 * item.quantity.max(0.0),
            sugar_g: 2.0 * item.quantity.max(0.0),
            saturated_fat_g: 1.0 * item.quantity.max(0.0),
            sodium_mg: 80.0 * item.quantity.max(0.0),
            potassium_mg: 150.0 * item.quantity.max(0.0),
            calcium_mg: 20.0 * item.quantity.max(0.0),
            iron_mg: 0.8 * item.quantity.max(0.0),
            magnesium_mg: 20.0 * item.quantity.max(0.0),
            zinc_mg: 0.5 * item.quantity.max(0.0),
            ..Nutrition::default()
        };
        acc.add(&estimated)
    })
}

pub async fn estimate_nutrition_from_foods_with_references(
    foods: &[FoodItem],
    references: Option<Arc<dyn FoodNutritionReferenceRepositoryPort>>,
) -> Nutrition {
    let Some(references) = references else {
        return estimate_nutrition_from_foods(foods);
    };

    let mut total = Nutrition::default();
    let mut fallback_foods = Vec::new();

    for food in foods {
        match references.find_reference_by_name(&food.name).await {
            Ok(Some(reference)) => {
                let factor = estimate_quantity_factor(
                    food.quantity,
                    &food.unit,
                    reference.basis_quantity,
                    &reference.basis_unit,
                );
                total = total.add(&reference.nutrition.scale(factor));
            }
            Ok(None) | Err(_) => fallback_foods.push(food.clone()),
        }
    }

    total.add(&estimate_nutrition_from_foods(&fallback_foods))
}

fn estimate_quantity_factor(
    quantity: f32,
    unit: &str,
    basis_quantity: f32,
    basis_unit: &str,
) -> f32 {
    let quantity = quantity.max(0.0);
    let basis_quantity = basis_quantity.max(1.0);
    let unit = unit.trim().to_lowercase();
    let basis_unit = basis_unit.trim().to_lowercase();

    if unit == basis_unit {
        return quantity / basis_quantity;
    }

    let grams = match unit.as_str() {
        "g" | "克" => Some(quantity),
        "kg" | "千克" | "公斤" => Some(quantity * 1000.0),
        _ => None,
    };

    let basis_grams = match basis_unit.as_str() {
        "g" | "克" => Some(basis_quantity),
        "kg" | "千克" | "公斤" => Some(basis_quantity * 1000.0),
        _ => None,
    };

    match (grams, basis_grams) {
        (Some(grams), Some(basis_grams)) if basis_grams > 0.0 => grams / basis_grams,
        _ => quantity / basis_quantity,
    }
}
