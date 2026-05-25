use std::sync::Arc;

use domain::{DietaryPreferences, UserHealthExpectation, UserId, UserPreferences, UserProfile};

use crate::app_error::{AppError, AppResult};
use crate::user::{
    UserHealthExpectationRepositoryPort, UserPreferencesRepositoryPort, UserProfileRepositoryPort,
};

#[derive(Clone)]
pub struct UserProfileQueryHandler {
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl UserProfileQueryHandler {
    pub fn new(user_profiles: Arc<dyn UserProfileRepositoryPort>) -> Self {
        Self { user_profiles }
    }

    pub async fn get_profile(&self, user_id: &UserId) -> AppResult<Option<UserProfile>> {
        self.user_profiles
            .find_profile(user_id)
            .await
            .map_err(AppError::upstream)
    }
}

#[derive(Debug, Clone)]
pub struct UserDietaryContext {
    pub profile: UserProfile,
    pub active_expectations: Vec<UserHealthExpectation>,
    pub preferences: UserPreferences,
}

impl UserDietaryContext {
    pub fn dietary_preferences(&self) -> &DietaryPreferences {
        &self.preferences.diet
    }
}

#[derive(Clone)]
pub struct UserDietaryContextQueryHandler {
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
    expectations: Arc<dyn UserHealthExpectationRepositoryPort>,
    preferences: Arc<dyn UserPreferencesRepositoryPort>,
}

impl UserDietaryContextQueryHandler {
    pub fn new(
        user_profiles: Arc<dyn UserProfileRepositoryPort>,
        expectations: Arc<dyn UserHealthExpectationRepositoryPort>,
        preferences: Arc<dyn UserPreferencesRepositoryPort>,
    ) -> Self {
        Self {
            user_profiles,
            expectations,
            preferences,
        }
    }

    pub async fn get_context(&self, user_id: &UserId) -> AppResult<Option<UserDietaryContext>> {
        let profile = self
            .user_profiles
            .find_profile(user_id)
            .await
            .map_err(AppError::upstream)?;

        let Some(profile) = profile else {
            return Ok(None);
        };

        let active_expectations = self
            .expectations
            .list_active_by_user(user_id)
            .await
            .map_err(AppError::upstream)?;

        let preferences = self
            .preferences
            .find_preferences(user_id)
            .await
            .map_err(AppError::upstream)?
            .unwrap_or_else(|| UserPreferences::new(user_id.clone()));

        Ok(Some(UserDietaryContext {
            profile,
            active_expectations,
            preferences,
        }))
    }
}

#[derive(Clone)]
pub struct UserHealthExpectationQueryHandler {
    repo: Arc<dyn UserHealthExpectationRepositoryPort>,
}

impl UserHealthExpectationQueryHandler {
    pub fn new(repo: Arc<dyn UserHealthExpectationRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn list_by_user(&self, user_id: &UserId) -> AppResult<Vec<UserHealthExpectation>> {
        self.repo
            .list_by_user(user_id)
            .await
            .map_err(AppError::upstream)
    }
}

#[derive(Clone)]
pub struct UserPreferencesQueryHandler {
    repo: Arc<dyn UserPreferencesRepositoryPort>,
}

impl UserPreferencesQueryHandler {
    pub fn new(repo: Arc<dyn UserPreferencesRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get_preferences(&self, user_id: &UserId) -> AppResult<Option<UserPreferences>> {
        self.repo
            .find_preferences(user_id)
            .await
            .map_err(AppError::upstream)
    }
}
