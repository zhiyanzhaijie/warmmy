//! This crate contains all shared fullstack server functions.
pub mod conversation;
mod impls;
pub mod meal;
pub mod user;

pub use conversation::{echo, echo_stream, get_session_history, list_user_sessions};
pub use user::{
    confirm_health_expectation, delete_health_expectation, get_user_preferences,
    list_health_expectations, update_user_preferences, upsert_health_expectation,
    HealthExpectationDto, UpdatePreferencesInput, UpsertHealthExpectationInput, UserPreferencesDto,
};
