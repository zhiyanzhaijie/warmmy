use std::sync::Arc;

use domain::{
    DietaryPreferences, DiningCompanion, UserHealthExpectation, UserId, UserPreferences,
    UserProfile,
};

use crate::app_error::{AppError, AppResult};
use crate::user::{
    DiningCompanionRepositoryPort, UserHealthExpectationRepositoryPort,
    UserPreferencesRepositoryPort, UserProfileRepositoryPort,
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

    pub async fn list_profiles(&self) -> AppResult<Vec<UserProfile>> {
        self.user_profiles
            .list_profiles()
            .await
            .map_err(AppError::upstream)
    }
}

#[derive(Debug, Clone)]
pub struct UserDietaryContext {
    pub profile: UserProfile,
    pub active_expectations: Vec<UserHealthExpectation>,
    pub preferences: UserPreferences,
    pub companions: Vec<DiningCompanion>,
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
    companions: Arc<dyn DiningCompanionRepositoryPort>,
}

impl UserDietaryContextQueryHandler {
    pub fn new(
        user_profiles: Arc<dyn UserProfileRepositoryPort>,
        expectations: Arc<dyn UserHealthExpectationRepositoryPort>,
        preferences: Arc<dyn UserPreferencesRepositoryPort>,
        companions: Arc<dyn DiningCompanionRepositoryPort>,
    ) -> Self {
        Self {
            user_profiles,
            expectations,
            preferences,
            companions,
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

        let companions = self
            .companions
            .list_companions(user_id)
            .await
            .map_err(AppError::upstream)?;

        Ok(Some(UserDietaryContext {
            profile,
            active_expectations,
            preferences,
            companions,
        }))
    }
}

#[derive(Clone)]
pub struct DiningCompanionQueryHandler {
    repo: Arc<dyn DiningCompanionRepositoryPort>,
}

impl DiningCompanionQueryHandler {
    pub fn new(repo: Arc<dyn DiningCompanionRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn list_companions(&self, owner_user_id: &UserId) -> AppResult<Vec<DiningCompanion>> {
        self.repo
            .list_companions(owner_user_id)
            .await
            .map_err(AppError::upstream)
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

#[derive(Debug, Clone)]
pub struct UserAIConfigSnapshot {
    pub providers: Vec<domain::UserAIProvider>,
    pub routes: Vec<domain::UserAIRoute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedAIModelConfig {
    pub provider_id: String,
    pub provider: domain::AIProviderKind,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub embedding_ndims: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AICapabilityStatus {
    pub capability: domain::AICapability,
    pub enabled: bool,
    pub configured: bool,
    pub reason: Option<String>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone)]
pub struct UserAIConfigQueryHandler {
    repo: Arc<dyn crate::user::UserAIConfigRepositoryPort>,
    secrets: Arc<dyn crate::user::SecretStorePort>,
}

impl UserAIConfigQueryHandler {
    pub fn new(
        repo: Arc<dyn crate::user::UserAIConfigRepositoryPort>,
        secrets: Arc<dyn crate::user::SecretStorePort>,
    ) -> Self {
        Self { repo, secrets }
    }

    pub async fn get_snapshot(&self, user_id: &UserId) -> AppResult<UserAIConfigSnapshot> {
        let providers = self
            .repo
            .list_providers(user_id)
            .await
            .map_err(AppError::upstream)?;
        let routes = self
            .repo
            .list_routes(user_id)
            .await
            .map_err(AppError::upstream)?;
        Ok(UserAIConfigSnapshot { providers, routes })
    }

    pub async fn resolve(
        &self,
        user_id: &UserId,
        capability: domain::AICapability,
    ) -> AppResult<ResolvedAIModelConfig> {
        self.resolve_optional(user_id, capability)
            .await?
            .ok_or_else(|| {
                AppError::validation(format!(
                    "AI capability is not configured: {}",
                    capability.as_str()
                ))
            })
    }

    pub async fn resolve_optional(
        &self,
        user_id: &UserId,
        capability: domain::AICapability,
    ) -> AppResult<Option<ResolvedAIModelConfig>> {
        let Some(route) = self
            .repo
            .find_route(user_id, capability)
            .await
            .map_err(AppError::upstream)?
        else {
            return Ok(None);
        };

        let provider = self
            .repo
            .list_providers(user_id)
            .await
            .map_err(AppError::upstream)?
            .into_iter()
            .find(|provider| provider.id == route.provider_id && provider.enabled);

        let Some(provider) = provider else {
            return Ok(None);
        };

        let Some(secret_ref) = provider.secret_ref.as_deref() else {
            return Ok(None);
        };

        let Some(api_key) = self
            .secrets
            .get_secret(secret_ref)
            .await
            .map_err(AppError::upstream)?
        else {
            return Ok(None);
        };

        if api_key.trim().is_empty() {
            return Ok(None);
        }

        Ok(Some(ResolvedAIModelConfig {
            provider_id: provider.id,
            provider: provider.kind,
            base_url: provider.base_url,
            api_key,
            model: route.model,
            embedding_ndims: route.embedding_ndims,
        }))
    }

    pub async fn list_statuses(&self, user_id: &UserId) -> AppResult<Vec<AICapabilityStatus>> {
        let snapshot = self.get_snapshot(user_id).await?;
        let mut statuses = Vec::new();
        for capability in [
            domain::AICapability::Chat,
            domain::AICapability::Embedding,
            domain::AICapability::Vision,
        ] {
            let route = snapshot
                .routes
                .iter()
                .filter(|route| route.capability == capability)
                .max_by_key(|route| (route.enabled, route.updated_at.clone()));
            let status = match route {
                Some(route) if !route.enabled => AICapabilityStatus {
                    capability,
                    enabled: false,
                    configured: true,
                    reason: Some("route disabled".to_string()),
                    provider_id: Some(route.provider_id.clone()),
                    model: Some(route.model.clone()),
                },
                Some(route) => {
                    let provider = snapshot
                        .providers
                        .iter()
                        .find(|provider| provider.id == route.provider_id);
                    match provider {
                        Some(provider) if !provider.enabled => AICapabilityStatus {
                            capability,
                            enabled: false,
                            configured: true,
                            reason: Some("provider disabled".to_string()),
                            provider_id: Some(route.provider_id.clone()),
                            model: Some(route.model.clone()),
                        },
                        Some(provider) if !provider.has_secret() => AICapabilityStatus {
                            capability,
                            enabled: false,
                            configured: true,
                            reason: Some("missing api key".to_string()),
                            provider_id: Some(route.provider_id.clone()),
                            model: Some(route.model.clone()),
                        },
                        Some(_) => AICapabilityStatus {
                            capability,
                            enabled: true,
                            configured: true,
                            reason: None,
                            provider_id: Some(route.provider_id.clone()),
                            model: Some(route.model.clone()),
                        },
                        None => AICapabilityStatus {
                            capability,
                            enabled: false,
                            configured: true,
                            reason: Some("provider not found".to_string()),
                            provider_id: Some(route.provider_id.clone()),
                            model: Some(route.model.clone()),
                        },
                    }
                }
                None => AICapabilityStatus {
                    capability,
                    enabled: false,
                    configured: false,
                    reason: Some("route missing".to_string()),
                    provider_id: None,
                    model: None,
                },
            };
            statuses.push(status);
        }
        Ok(statuses)
    }
}
