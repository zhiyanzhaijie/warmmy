use std::sync::Arc;

use domain::{
    AppPreferences, DietaryPreferences, DiningCompanion, DiningCompanionId, HealthExpectationId,
    HealthExpectationKind, HealthExpectationStatus, PreferenceConfidence, UserHealthExpectation,
    UserId, UserPreferences, UserProfile,
};

use crate::app_error::{AppError, AppResult};
use crate::user::{
    DiningCompanionRepositoryPort, UserHealthExpectationRepositoryPort,
    UserPreferencesRepositoryPort, UserProfileRepositoryPort,
};

#[derive(Debug, Clone)]
pub struct EnsureUserProfileCommand {
    pub user_id: UserId,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct SaveUserProfileCommand {
    pub user_id: UserId,
    pub display_name: String,
    pub introduction: String,
    pub gender: Option<String>,
    pub age: Option<u8>,
}

#[derive(Clone)]
pub struct UserProfileCommandHandler {
    repo: Arc<dyn UserProfileRepositoryPort>,
}

impl UserProfileCommandHandler {
    pub fn new(repo: Arc<dyn UserProfileRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn ensure_profile(&self, input: EnsureUserProfileCommand) -> AppResult<UserProfile> {
        if let Some(profile) = self
            .repo
            .find_profile(&input.user_id)
            .await
            .map_err(AppError::upstream)?
        {
            return Ok(profile);
        }

        let profile = UserProfile::new(input.user_id.as_str(), input.display_name);

        self.repo
            .save_profile(&profile)
            .await
            .map_err(AppError::upstream)?;

        Ok(profile)
    }

    pub async fn save_profile(&self, input: SaveUserProfileCommand) -> AppResult<UserProfile> {
        let profile = UserProfile {
            id: input.user_id,
            display_name: input.display_name.trim().to_string(),
            introduction: input.introduction.trim().to_string(),
            gender: input.gender.and_then(|value| {
                let value = value.trim().to_string();
                (!value.is_empty()).then_some(value)
            }),
            age: input.age,
        };

        if profile.display_name.is_empty() {
            return Err(AppError::validation("display name is empty"));
        }

        self.repo
            .save_profile(&profile)
            .await
            .map_err(AppError::upstream)?;

        Ok(profile)
    }
}

#[derive(Debug, Clone)]
pub struct ProposeHealthExpectationByAgentCommand {
    pub expectation: UserHealthExpectation,
}

#[derive(Debug, Clone)]
pub struct ConfirmHealthExpectationCommand {
    pub user_id: UserId,
    pub expectation_id: HealthExpectationId,
}

#[derive(Debug, Clone)]
pub struct DeleteHealthExpectationCommand {
    pub user_id: UserId,
    pub expectation_id: HealthExpectationId,
}

#[derive(Debug, Clone)]
pub struct UpdateUserPreferencesCommand {
    pub user_id: UserId,
    pub app: AppPreferences,
    pub diet: DietaryPreferences,
}

#[derive(Debug, Clone)]
pub struct SaveDiningCompanionCommand {
    pub id: DiningCompanionId,
    pub owner_user_id: UserId,
    pub display_name: String,
    pub relationship: Option<String>,
    pub introduction: String,
    pub diet: DietaryPreferences,
    pub health_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteDiningCompanionCommand {
    pub owner_user_id: UserId,
    pub companion_id: DiningCompanionId,
}

#[derive(Clone)]
pub struct DiningCompanionCommandHandler {
    repo: Arc<dyn DiningCompanionRepositoryPort>,
    profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl DiningCompanionCommandHandler {
    pub fn new(
        repo: Arc<dyn DiningCompanionRepositoryPort>,
        profiles: Arc<dyn UserProfileRepositoryPort>,
    ) -> Self {
        Self { repo, profiles }
    }

    pub async fn save(&self, input: SaveDiningCompanionCommand) -> AppResult<DiningCompanion> {
        let profile = self
            .profiles
            .find_profile(&input.owner_user_id)
            .await
            .map_err(AppError::upstream)?;

        if profile.is_none() {
            return Err(AppError::NotFound(format!(
                "owner profile not found for {}",
                input.owner_user_id.as_str()
            )));
        }

        let display_name = input.display_name.trim().to_string();
        if display_name.is_empty() {
            return Err(AppError::validation("companion display name is empty"));
        }

        let companion = DiningCompanion {
            id: input.id,
            owner_user_id: input.owner_user_id,
            display_name,
            relationship: input.relationship.and_then(|value| {
                let value = value.trim().to_string();
                (!value.is_empty()).then_some(value)
            }),
            introduction: input.introduction.trim().to_string(),
            diet: input.diet,
            health_notes: input
                .health_notes
                .into_iter()
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        };

        self.repo
            .save_companion(&companion)
            .await
            .map_err(AppError::upstream)?;

        Ok(companion)
    }

    pub async fn delete(&self, input: DeleteDiningCompanionCommand) -> AppResult<()> {
        self.repo
            .delete_companion(&input.owner_user_id, &input.companion_id)
            .await
            .map_err(AppError::upstream)
    }
}

#[derive(Clone)]
pub struct UserHealthExpectationCommandHandler {
    repo: Arc<dyn UserHealthExpectationRepositoryPort>,
    profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl UserHealthExpectationCommandHandler {
    pub fn new(
        repo: Arc<dyn UserHealthExpectationRepositoryPort>,
        profiles: Arc<dyn UserProfileRepositoryPort>,
    ) -> Self {
        Self { repo, profiles }
    }

    pub async fn propose_by_agent(
        &self,
        input: ProposeHealthExpectationByAgentCommand,
    ) -> AppResult<()> {
        if input.expectation.user_id.as_str().is_empty() {
            return Err(AppError::validation("user id is empty"));
        }

        self.ensure_profile_exists(&input.expectation.user_id)
            .await?;

        self.repo
            .save_expectation(&input.expectation)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn confirm(&self, input: ConfirmHealthExpectationCommand) -> AppResult<()> {
        self.ensure_profile_exists(&input.user_id).await?;

        let expectations = self
            .repo
            .list_by_user(&input.user_id)
            .await
            .map_err(AppError::upstream)?;

        let mut expectation = expectations
            .into_iter()
            .find(|item| item.id == input.expectation_id)
            .ok_or_else(|| AppError::NotFound("health expectation".to_string()))?;

        expectation.status = HealthExpectationStatus::Active;
        self.repo
            .save_expectation(&expectation)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn delete(&self, input: DeleteHealthExpectationCommand) -> AppResult<()> {
        self.ensure_profile_exists(&input.user_id).await?;

        self.repo
            .delete_expectation(&input.user_id, &input.expectation_id)
            .await
            .map_err(AppError::upstream)
    }

    async fn ensure_profile_exists(&self, user_id: &UserId) -> AppResult<()> {
        let profile = self
            .profiles
            .find_profile(user_id)
            .await
            .map_err(AppError::upstream)?;

        if profile.is_none() {
            return Err(AppError::NotFound(format!(
                "user profile not found for {}",
                user_id.as_str()
            )));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct UserPreferencesCommandHandler {
    repo: Arc<dyn UserPreferencesRepositoryPort>,
    profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl UserPreferencesCommandHandler {
    pub fn new(
        repo: Arc<dyn UserPreferencesRepositoryPort>,
        profiles: Arc<dyn UserProfileRepositoryPort>,
    ) -> Self {
        Self { repo, profiles }
    }

    pub async fn update(&self, input: UpdateUserPreferencesCommand) -> AppResult<()> {
        let profile = self
            .profiles
            .find_profile(&input.user_id)
            .await
            .map_err(AppError::upstream)?;

        if profile.is_none() {
            return Err(AppError::NotFound(format!(
                "user profile not found for {}",
                input.user_id.as_str()
            )));
        }

        let preferences = UserPreferences {
            user_id: input.user_id,
            app: input.app,
            diet: input.diet,
        };

        self.repo
            .save_preferences(&preferences)
            .await
            .map_err(AppError::upstream)
    }
}

pub fn explicit_cuisine(cuisine: impl Into<String>) -> domain::CuisinePreference {
    domain::CuisinePreference {
        cuisine: cuisine.into(),
        confidence: PreferenceConfidence::Explicit,
    }
}

pub fn inferred_health_expectation(
    id: HealthExpectationId,
    user_id: UserId,
    title: impl Into<String>,
    summary: impl Into<String>,
    kind: HealthExpectationKind,
    session_id: impl Into<String>,
    now_rfc3339: impl Into<String>,
) -> UserHealthExpectation {
    let now_rfc3339 = now_rfc3339.into();
    UserHealthExpectation {
        id,
        user_id,
        title: title.into(),
        summary: summary.into(),
        kind,
        status: HealthExpectationStatus::Proposed,
        source: domain::ExpectationSource::AgentProposed {
            session_id: session_id.into(),
        },
        priority: 50,
        created_at: now_rfc3339.clone(),
        updated_at: now_rfc3339,
    }
}
