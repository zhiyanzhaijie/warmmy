//! This crate contains all shared fullstack server functions.
#[cfg(all(feature = "server", feature = "local"))]
compile_error!("api features `server` and `local` cannot be enabled together");

pub mod conversation;
mod impls;
#[cfg(feature = "local")]
pub mod local_state;
pub mod meal;
pub mod user;

pub use conversation::{
    delete_ephemeral_image, echo, echo_stream, get_session_history, list_user_sessions,
    store_ephemeral_image,
};
pub use meal::{
    confirm_pending_meal, finalize_and_summarize_meal_day, get_meal_day_summary,
    list_pending_meals, preview_pending_meal, reject_pending_meal, ConfirmPendingMealInput,
    FoodItemDTO, MealDaySummaryDTO, NutritionDTO, PendingMealLogDTO,
};
pub use user::{
    confirm_health_expectation, create_user_profile, delete_dining_companion,
    delete_health_expectation, delete_user_ai_provider, get_user_ai_config, get_user_preferences,
    get_user_profile, list_dining_companions, list_health_expectations, list_user_profiles,
    save_dining_companion, save_user_ai_provider, save_user_ai_route, save_user_profile,
    update_user_preferences, upsert_health_expectation, DiningCompanionDTO, HealthExpectationDTO,
    SaveDiningCompanionInput, SaveUserAIProviderInput, SaveUserAIRouteInput, SaveUserProfileInput,
    UpdatePreferencesInput, UpsertHealthExpectationInput, UserAICapabilityStatusDTO,
    UserAIConfigDTO, UserAIProviderDTO, UserAIRouteDTO, UserPreferencesDTO, UserProfileDTO,
};
