use domain::{MealRecord, UserProfile};

pub fn build_meal_advice_prompt(
    profile: &UserProfile,
    meal: &MealRecord,
    recent_dialogue: &[String],
    knowledge_hits: &[String],
) -> String {
    format!(
        "User: {}. Goal: {}. Allergies: {:?}. Meal foods: {:?}. Nutrition: {:?}. Recent dialogue count: {}. Knowledge hits count: {}. Please provide concise dietary advice and the next meal suggestion.",
        profile.display_name,
        profile.health_goal.as_str(),
        profile.allergies,
        meal.foods.iter().map(|f| &f.name).collect::<Vec<_>>(),
        meal.nutrition,
        recent_dialogue.len(),
        knowledge_hits.len()
    )
}
