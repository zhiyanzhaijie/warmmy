use domain::{FoodItem, Nutrition};

pub fn estimate_nutrition_from_foods(foods: &[FoodItem]) -> Nutrition {
    foods.iter().fold(Nutrition::default(), |acc, item| {
        let estimated = Nutrition {
            calories: 120.0 * item.quantity.max(0.0),
            protein_g: 4.0 * item.quantity.max(0.0),
            fat_g: 3.0 * item.quantity.max(0.0),
            carbs_g: 12.0 * item.quantity.max(0.0),
        };
        acc.add(&estimated)
    })
}
