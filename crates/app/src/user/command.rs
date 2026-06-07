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

#[derive(Debug, Clone)]
pub struct SaveUserAIProviderCommand {
    pub user_id: UserId,
    pub provider_id: Option<String>,
    pub kind: domain::AIProviderKind,
    pub name: String,
    pub base_url: String,
    pub api_key_ref: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SaveUserApiKeyCommand {
    pub user_id: UserId,
    pub api_key_id: Option<String>,
    pub name: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteUserApiKeyCommand {
    pub user_id: UserId,
    pub api_key_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteUserAIProviderCommand {
    pub user_id: UserId,
    pub provider_id: String,
}
#[derive(Debug, Clone)]
pub struct DeleteUserAIRouteCommand {
    pub user_id: UserId,
    pub route_id: String,
}

#[derive(Debug, Clone)]
pub struct SaveUserAIRouteCommand {
    pub user_id: UserId,
    pub route_id: Option<String>,
    pub capability: domain::AICapability,
    pub provider_id: String,
    pub model: String,
    pub embedding_ndims: Option<usize>,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct UserAIConfigCommandHandler {
    repo: Arc<dyn crate::user::UserAIConfigRepositoryPort>,
    secrets: Arc<dyn crate::user::SecretStorePort>,
    profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl UserAIConfigCommandHandler {
    pub fn new(
        repo: Arc<dyn crate::user::UserAIConfigRepositoryPort>,
        secrets: Arc<dyn crate::user::SecretStorePort>,
        profiles: Arc<dyn UserProfileRepositoryPort>,
    ) -> Self {
        Self {
            repo,
            secrets,
            profiles,
        }
    }

    pub async fn save_provider(
        &self,
        input: SaveUserAIProviderCommand,
    ) -> AppResult<domain::UserAIProvider> {
        self.ensure_profile_exists(&input.user_id).await?;

        let now = chrono::Utc::now().to_rfc3339();
        let provider_id = input.provider_id.unwrap_or_else(|| {
            format!(
                "ai-provider-{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            )
        });
        let name = input.name.trim().to_string();
        let base_url = input.base_url.trim().to_string();

        if name.is_empty() {
            return Err(AppError::validation("provider name is empty"));
        }
        if base_url.is_empty() {
            return Err(AppError::validation("provider base_url is empty"));
        }

        let existing = self
            .repo
            .list_providers(&input.user_id)
            .await
            .map_err(AppError::upstream)?
            .into_iter()
            .find(|item| item.id == provider_id);

        let api_keys = self
            .secrets
            .list_user_api_keys(&input.user_id)
            .await
            .map_err(AppError::upstream)?;
        let secret_ref = match input
            .api_key_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(secret_ref) => Some(
                api_keys
                    .iter()
                    .find(|item| item.secret_ref == secret_ref)
                    .map(|item| item.secret_ref.clone())
                    .ok_or_else(|| AppError::validation(format!("api key not found: {secret_ref}")))?,
            ),
            None => existing.and_then(|provider| provider.secret_ref),
        };

        let provider = domain::UserAIProvider {
            id: provider_id,
            user_id: input.user_id,
            kind: input.kind,
            name,
            base_url,
            secret_ref,
            enabled: input.enabled,
            updated_at: now,
        };

        self.repo
            .save_provider(&provider)
            .await
            .map_err(AppError::upstream)?;

        Ok(provider)
    }

    pub async fn delete_provider(&self, input: DeleteUserAIProviderCommand) -> AppResult<()> {
        self.ensure_profile_exists(&input.user_id).await?;
        self.repo
            .delete_provider(&input.user_id, &input.provider_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn delete_route(&self, input: DeleteUserAIRouteCommand) -> AppResult<()> {
        self.ensure_profile_exists(&input.user_id).await?;
        self.repo
            .delete_route(&input.user_id, &input.route_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn save_route(
        &self,
        input: SaveUserAIRouteCommand,
    ) -> AppResult<domain::UserAIRoute> {
        self.ensure_profile_exists(&input.user_id).await?;
        let model = input.model.trim().to_string();
        if model.is_empty() {
            return Err(AppError::validation("model is empty"));
        }

        let provider_exists = self
            .repo
            .list_providers(&input.user_id)
            .await
            .map_err(AppError::upstream)?
            .into_iter()
            .any(|provider| provider.id == input.provider_id);

        if !provider_exists {
            return Err(AppError::NotFound(format!(
                "ai provider not found: {}",
                input.provider_id
            )));
        }
        let route_id = input.route_id.unwrap_or_else(|| {
            format!(
                "ai-route-{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            )
        });
        let updated_at = chrono::Utc::now().to_rfc3339();

        if input.enabled {
            let routes = self
                .repo
                .list_routes(&input.user_id)
                .await
                .map_err(AppError::upstream)?;
            for existing in routes {
                if existing.capability == input.capability
                    && existing.id != route_id
                    && existing.enabled
                {
                    let mut disabled = existing;
                    disabled.enabled = false;
                    disabled.updated_at = updated_at.clone();
                    self.repo
                        .save_route(&disabled)
                        .await
                        .map_err(AppError::upstream)?;
                }
            }
        }

        let route = domain::UserAIRoute {
            id: route_id,
            user_id: input.user_id,
            capability: input.capability,
            provider_id: input.provider_id,
            model,
            embedding_ndims: input.embedding_ndims,
            enabled: input.enabled,
            updated_at,
        };

        self.repo
            .save_route(&route)
            .await
            .map_err(AppError::upstream)?;

        Ok(route)
    }

    pub async fn save_api_key(&self, input: SaveUserApiKeyCommand) -> AppResult<domain::UserApiKey> {
        self.ensure_profile_exists(&input.user_id).await?;

        let now = chrono::Utc::now().to_rfc3339();
        let api_key_id = input.api_key_id.unwrap_or_else(|| {
            format!(
                "ai-key-{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            )
        });
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::validation("api key name is empty"));
        }

        let existing = self
            .secrets
            .list_user_api_keys(&input.user_id)
            .await
            .map_err(AppError::upstream)?
            .into_iter()
            .find(|item| item.id == api_key_id);
        let secret_ref = format!("secret:user:{}:api_key:{api_key_id}", input.user_id.as_str());
        let secret_value = match input.api_key {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => {
                let existing = existing.ok_or_else(|| {
                    AppError::validation("new api key requires a plaintext value")
                })?;
                self.secrets
                    .get_secret(&existing.secret_ref)
                    .await
                    .map_err(AppError::upstream)?
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| AppError::validation("existing api key payload is missing"))?
            }
        };

        let api_key = domain::UserApiKey {
            id: api_key_id,
            user_id: input.user_id,
            name,
            secret_ref,
            updated_at: now,
        };

        self.secrets
            .save_user_api_key(&api_key, &secret_value)
            .await
            .map_err(AppError::upstream)?;

        Ok(api_key)
    }

    pub async fn delete_api_key(&self, input: DeleteUserApiKeyCommand) -> AppResult<()> {
        self.ensure_profile_exists(&input.user_id).await?;

        let api_key = self
            .secrets
            .list_user_api_keys(&input.user_id)
            .await
            .map_err(AppError::upstream)?
            .into_iter()
            .find(|item| item.id == input.api_key_id)
            .ok_or_else(|| AppError::NotFound(format!("api key not found: {}", input.api_key_id)))?;

        let providers = self
            .repo
            .list_providers(&input.user_id)
            .await
            .map_err(AppError::upstream)?;
        if providers
            .iter()
            .any(|provider| provider.secret_ref.as_deref() == Some(api_key.secret_ref.as_str()))
        {
            return Err(AppError::validation("api key is still referenced by a provider"));
        }

        self.secrets
            .delete_user_api_key(&input.user_id, &input.api_key_id)
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
