use crate::user::UserDietaryContext;
use domain::MealRecord;

pub fn build_meal_advice_prompt(
    context: &UserDietaryContext,
    meal: &MealRecord,
    recent_dialogue: &[String],
    knowledge_hits: &[String],
) -> String {
    let expectation_summary = if context.active_expectations.is_empty() {
        context.profile.health_goal.as_str().to_string()
    } else {
        context
            .active_expectations
            .iter()
            .map(|item| item.title.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let preferred_cuisines = context
        .dietary_preferences()
        .preferred_cuisines
        .iter()
        .map(|item| item.cuisine.as_str())
        .collect::<Vec<_>>();

    let avoided_cuisines = context
        .dietary_preferences()
        .avoided_cuisines
        .iter()
        .map(|item| item.cuisine.as_str())
        .collect::<Vec<_>>();

    format!(
        "User: {}. Goals: {}. Allergies: {:?}. Preferred cuisines: {:?}. Avoided cuisines: {:?}. Meal foods: {:?}. Nutrition: {:?}. Recent dialogue count: {}. Knowledge hits count: {}. Please provide concise dietary advice and the next meal suggestion.",
        context.profile.display_name,
        expectation_summary,
        context.profile.allergies,
        preferred_cuisines,
        avoided_cuisines,
        meal.foods.iter().map(|f| &f.name).collect::<Vec<_>>(),
        meal.nutrition,
        recent_dialogue.len(),
        knowledge_hits.len()
    )
}
