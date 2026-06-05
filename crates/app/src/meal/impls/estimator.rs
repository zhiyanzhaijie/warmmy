use domain::{FoodItem, FoodNutritionReference, Nutrition};

pub fn estimate_nutrition_from_foods_with_references(
    foods: &[FoodItem],
    references: &[Option<FoodNutritionReference>],
) -> Nutrition {
    let mut total = Nutrition::default();

    for (food, reference) in foods.iter().zip(references.iter()) {
        if let Some(reference) = reference {
            let factor = estimate_quantity_factor(
                food.estimated_grams,
                food.quantity,
                &food.unit,
                reference.basis_quantity,
                &reference.basis_unit,
            );
            total = total.add(&reference.nutrition.scale(factor));
        }
    }

    total
}

fn estimate_quantity_factor(
    estimated_grams: Option<f32>,
    quantity: f32,
    unit: &str,
    basis_quantity: f32,
    basis_unit: &str,
) -> f32 {
    let quantity = quantity.max(0.0);
    let basis_quantity = basis_quantity.max(1.0);
    let unit = unit.trim().to_lowercase();
    let basis_unit = basis_unit.trim().to_lowercase();

    let basis_grams = unit_grams(basis_quantity, &basis_unit);
    if let (Some(estimated_grams), Some(basis_grams)) = (estimated_grams, basis_grams) {
        if estimated_grams > 0.0 && basis_grams > 0.0 {
            return estimated_grams / basis_grams;
        }
    }

    if unit == basis_unit {
        return quantity / basis_quantity;
    }

    let grams = unit_grams(quantity, &unit);

    match (grams, basis_grams) {
        (Some(grams), Some(basis_grams)) if basis_grams > 0.0 => grams / basis_grams,
        _ => quantity / basis_quantity,
    }
}

fn unit_grams(quantity: f32, unit: &str) -> Option<f32> {
    match unit {
        "g" | "克" => Some(quantity),
        "kg" | "千克" | "公斤" => Some(quantity * 1000.0),
        _ => None,
    }
}
