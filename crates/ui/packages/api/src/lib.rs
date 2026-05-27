//! This crate contains all shared fullstack server functions.
pub mod conversation;
mod impls;
pub mod meal;
pub mod user;

pub use conversation::{echo, echo_stream, get_session_history, list_user_sessions};
pub use meal::{
    confirm_pending_meal, finalize_and_summarize_meal_day, get_meal_day_summary,
    list_pending_meals, preview_pending_meal, reject_pending_meal, ConfirmPendingMealInput,
    FoodItemDTO, MealDaySummaryDTO, NutritionDTO, PendingMealLogDTO,
};
pub use user::{
    confirm_health_expectation, create_user_profile, delete_dining_companion,
    delete_health_expectation, get_user_preferences, get_user_profile, list_dining_companions,
    list_health_expectations, list_user_profiles, save_dining_companion, save_user_profile,
    update_user_preferences, upsert_health_expectation, DiningCompanionDTO, HealthExpectationDTO,
    SaveDiningCompanionInput, SaveUserProfileInput, UpdatePreferencesInput,
    UpsertHealthExpectationInput, UserPreferencesDTO, UserProfileDTO,
};
