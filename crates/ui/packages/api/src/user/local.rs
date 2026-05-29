use crate::impls::error::api_error;
use dioxus::prelude::*;

const DEFAULT_DISPLAY_NAME: &str = "屋主";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserProfileDTO {
    pub id: String,
    pub display_name: String,
    pub introduction: String,
    pub gender: Option<String>,
    pub age: Option<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HealthExpectationDTO {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub kind: String,
    pub status: String,
    pub priority: u8,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserPreferencesDTO {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub preferred_cuisines: Vec<String>,
    pub avoided_cuisines: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DiningCompanionDTO {
    pub id: String,
    pub display_name: String,
    pub relationship: Option<String>,
    pub introduction: String,
    pub preferred_cuisines: Vec<String>,
    pub avoided_cuisines: Vec<String>,
    pub health_notes: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserAIProviderDTO {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub base_url: String,
    pub has_api_key: bool,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserAIRouteDTO {
    pub id: String,
    pub capability: String,
    pub provider_id: String,
    pub model: String,
    pub embedding_ndims: Option<usize>,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserAICapabilityStatusDTO {
    pub capability: String,
    pub enabled: bool,
    pub configured: bool,
    pub reason: Option<String>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserAIConfigDTO {
    pub providers: Vec<UserAIProviderDTO>,
    pub routes: Vec<UserAIRouteDTO>,
    pub statuses: Vec<UserAICapabilityStatusDTO>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveUserAIProviderInput {
    pub id: Option<String>,
    pub kind: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveUserAIRouteInput {
    pub id: Option<String>,
    pub capability: String,
    pub provider_id: String,
    pub model: String,
    pub embedding_ndims: Option<usize>,
    pub enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveUserProfileInput {
    pub id: String,
    pub display_name: String,
    pub introduction: String,
    pub gender: Option<String>,
    pub age: Option<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UpsertHealthExpectationInput {
    pub id: Option<String>,
    pub title: String,
    pub summary: String,
    pub kind: String,
    pub priority: u8,
    pub status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UpdatePreferencesInput {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub preferred_cuisines: Vec<String>,
    pub avoided_cuisines: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveDiningCompanionInput {
    pub id: Option<String>,
    pub display_name: String,
    pub relationship: Option<String>,
    pub introduction: String,
    pub preferred_cuisines: Vec<String>,
    pub avoided_cuisines: Vec<String>,
    pub health_notes: Vec<String>,
}

pub async fn get_user_profile(user_id: String) -> Result<UserProfileDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let profile = state
        .0
        .user
        .query
        .get_profile(&user_id)
        .await
        .map_err(api_error)?
        .ok_or_else(|| ServerFnError::ServerError {
            message: format!("user profile not found for {}", user_id.as_str()),
            code: 404,
            details: None,
        })?;

    Ok(profile_to_dto(profile))
}

pub async fn list_user_profiles() -> Result<Vec<UserProfileDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let mut profiles = state
        .0
        .user
        .query
        .list_profiles()
        .await
        .map_err(api_error)?
        .into_iter()
        .map(profile_to_dto)
        .collect::<Vec<_>>();

    profiles.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(profiles)
}

pub async fn create_user_profile() -> Result<UserProfileDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let profiles = state
        .0
        .user
        .query
        .list_profiles()
        .await
        .map_err(api_error)?;
    let next_index = profiles
        .iter()
        .filter_map(|profile| profile.id.as_str().parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let user_id = domain::UserId::new_unchecked(next_index.to_string());
    let profile = state
        .0
        .user
        .command
        .save_profile(app::user::SaveUserProfileCommand {
            user_id,
            display_name: DEFAULT_DISPLAY_NAME.to_string(),
            introduction: String::new(),
            gender: None,
            age: None,
        })
        .await
        .map_err(api_error)?;

    Ok(profile_to_dto(profile))
}

pub async fn save_user_profile(
    input: SaveUserProfileInput,
) -> Result<UserProfileDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&input.id)?;
    let profile = state
        .0
        .user
        .command
        .save_profile(app::user::SaveUserProfileCommand {
            user_id,
            display_name: input.display_name,
            introduction: input.introduction,
            gender: input.gender,
            age: input.age,
        })
        .await
        .map_err(api_error)?;

    Ok(profile_to_dto(profile))
}

pub async fn get_user_preferences(user_id: String) -> Result<UserPreferencesDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let preferences = state
        .0
        .user
        .preferences_query
        .get_preferences(&user_id)
        .await
        .map_err(api_error)?
        .unwrap_or_else(|| domain::UserPreferences::new(user_id.clone()));

    Ok(UserPreferencesDTO {
        theme: preferences.app.theme.map(theme_to_string),
        language: preferences.app.language,
        preferred_cuisines: preferences
            .diet
            .preferred_cuisines
            .into_iter()
            .map(|item| item.cuisine)
            .collect(),
        avoided_cuisines: preferences
            .diet
            .avoided_cuisines
            .into_iter()
            .map(|item| item.cuisine)
            .collect(),
    })
}

pub async fn update_user_preferences(
    user_id: String,
    input: UpdatePreferencesInput,
) -> Result<UserPreferencesDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let command = app::user::UpdateUserPreferencesCommand {
        user_id: user_id.clone(),
        app: domain::AppPreferences {
            theme: input.theme.as_deref().and_then(parse_theme),
            language: input
                .language
                .clone()
                .filter(|value| !value.trim().is_empty()),
        },
        diet: domain::DietaryPreferences {
            preferred_cuisines: input
                .preferred_cuisines
                .into_iter()
                .filter(|item| !item.trim().is_empty())
                .map(app::user::explicit_cuisine)
                .collect(),
            avoided_cuisines: input
                .avoided_cuisines
                .into_iter()
                .filter(|item| !item.trim().is_empty())
                .map(app::user::explicit_cuisine)
                .collect(),
        },
    };

    state
        .0
        .user
        .preferences_command
        .update(command)
        .await
        .map_err(api_error)?;

    get_user_preferences(user_id.to_string()).await
}

pub async fn list_health_expectations(
    user_id: String,
) -> Result<Vec<HealthExpectationDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let expectations = state
        .0
        .user
        .expectation_query
        .list_by_user(&user_id)
        .await
        .map_err(api_error)?;

    Ok(expectations.into_iter().map(expectation_to_dto).collect())
}

pub async fn upsert_health_expectation(
    user_id: String,
    input: UpsertHealthExpectationInput,
) -> Result<Vec<HealthExpectationDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let now = chrono::Utc::now().to_rfc3339();
    let id = input.id.unwrap_or_else(|| {
        format!(
            "hx-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        )
    });

    let expectation = domain::UserHealthExpectation {
        id: domain::HealthExpectationId::new_unchecked(id),
        user_id: user_id.clone(),
        title: input.title.trim().to_string(),
        summary: input.summary.trim().to_string(),
        kind: parse_kind(&input.kind),
        status: parse_status(&input.status)?,
        source: domain::ExpectationSource::UserExplicit,
        priority: input.priority,
        created_at: now.clone(),
        updated_at: now,
    };

    state
        .0
        .user
        .expectation_command
        .propose_by_agent(app::user::ProposeHealthExpectationByAgentCommand { expectation })
        .await
        .map_err(api_error)?;

    list_health_expectations(user_id.to_string()).await
}

pub async fn confirm_health_expectation(
    user_id: String,
    expectation_id: String,
) -> Result<Vec<HealthExpectationDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    state
        .0
        .user
        .expectation_command
        .confirm(app::user::ConfirmHealthExpectationCommand {
            user_id: user_id.clone(),
            expectation_id: domain::HealthExpectationId::new_unchecked(expectation_id),
        })
        .await
        .map_err(api_error)?;

    list_health_expectations(user_id.to_string()).await
}

pub async fn delete_health_expectation(
    user_id: String,
    expectation_id: String,
) -> Result<Vec<HealthExpectationDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    state
        .0
        .user
        .expectation_command
        .delete(app::user::DeleteHealthExpectationCommand {
            user_id: user_id.clone(),
            expectation_id: domain::HealthExpectationId::new_unchecked(expectation_id),
        })
        .await
        .map_err(api_error)?;

    list_health_expectations(user_id.to_string()).await
}

pub async fn list_dining_companions(
    user_id: String,
) -> Result<Vec<DiningCompanionDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let companions = state
        .0
        .user
        .companion_query
        .list_companions(&user_id)
        .await
        .map_err(api_error)?;

    Ok(companions.into_iter().map(companion_to_dto).collect())
}

pub async fn save_dining_companion(
    user_id: String,
    input: SaveDiningCompanionInput,
) -> Result<Vec<DiningCompanionDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let id = input.id.unwrap_or_else(|| {
        format!(
            "dc-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        )
    });

    state
        .0
        .user
        .companion_command
        .save(app::user::SaveDiningCompanionCommand {
            id: domain::DiningCompanionId::new_unchecked(id),
            owner_user_id: user_id.clone(),
            display_name: input.display_name,
            relationship: input.relationship,
            introduction: input.introduction,
            diet: domain::DietaryPreferences {
                preferred_cuisines: input
                    .preferred_cuisines
                    .into_iter()
                    .filter(|item| !item.trim().is_empty())
                    .map(app::user::explicit_cuisine)
                    .collect(),
                avoided_cuisines: input
                    .avoided_cuisines
                    .into_iter()
                    .filter(|item| !item.trim().is_empty())
                    .map(app::user::explicit_cuisine)
                    .collect(),
            },
            health_notes: input.health_notes,
        })
        .await
        .map_err(api_error)?;

    list_dining_companions(user_id.to_string()).await
}

pub async fn delete_dining_companion(
    user_id: String,
    companion_id: String,
) -> Result<Vec<DiningCompanionDTO>, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    state
        .0
        .user
        .companion_command
        .delete(app::user::DeleteDiningCompanionCommand {
            owner_user_id: user_id.clone(),
            companion_id: domain::DiningCompanionId::new_unchecked(companion_id),
        })
        .await
        .map_err(api_error)?;

    list_dining_companions(user_id.to_string()).await
}

pub async fn get_user_ai_config(user_id: String) -> Result<UserAIConfigDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let snapshot = state
        .0
        .user
        .ai_config_query
        .get_snapshot(&user_id)
        .await
        .map_err(api_error)?;
    let statuses = state
        .0
        .user
        .ai_config_query
        .list_statuses(&user_id)
        .await
        .map_err(api_error)?;

    Ok(UserAIConfigDTO {
        providers: snapshot
            .providers
            .into_iter()
            .map(ai_provider_to_dto)
            .collect(),
        routes: snapshot.routes.into_iter().map(ai_route_to_dto).collect(),
        statuses: statuses.into_iter().map(ai_status_to_dto).collect(),
    })
}

pub async fn save_user_ai_provider(
    user_id: String,
    input: SaveUserAIProviderInput,
) -> Result<UserAIConfigDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let kind = parse_ai_provider_kind(&input.kind)?;
    state
        .0
        .user
        .ai_config_command
        .save_provider(app::user::SaveUserAIProviderCommand {
            user_id: user_id.clone(),
            provider_id: input.id,
            kind,
            name: input.name,
            base_url: input.base_url,
            api_key: input.api_key,
            enabled: input.enabled,
        })
        .await
        .map_err(api_error)?;

    get_user_ai_config(user_id.to_string()).await
}

pub async fn delete_user_ai_provider(
    user_id: String,
    provider_id: String,
) -> Result<UserAIConfigDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    state
        .0
        .user
        .ai_config_command
        .delete_provider(app::user::DeleteUserAIProviderCommand {
            user_id: user_id.clone(),
            provider_id,
        })
        .await
        .map_err(api_error)?;

    get_user_ai_config(user_id.to_string()).await
}

pub async fn save_user_ai_route(
    user_id: String,
    input: SaveUserAIRouteInput,
) -> Result<UserAIConfigDTO, ServerFnError> {
    let state = crate::local_state::state().await?;
    let user_id = parse_user_id(&user_id)?;
    let capability = parse_ai_capability(&input.capability)?;
    state
        .0
        .user
        .ai_config_command
        .save_route(app::user::SaveUserAIRouteCommand {
            user_id: user_id.clone(),
            route_id: input.id,
            capability,
            provider_id: input.provider_id,
            model: input.model,
            embedding_ndims: input.embedding_ndims,
            enabled: input.enabled,
        })
        .await
        .map_err(api_error)?;

    get_user_ai_config(user_id.to_string()).await
}

fn ai_provider_to_dto(provider: domain::UserAIProvider) -> UserAIProviderDTO {
    let has_api_key = provider.has_secret();
    UserAIProviderDTO {
        id: provider.id,
        kind: provider.kind.as_str().to_string(),
        name: provider.name,
        base_url: provider.base_url,
        has_api_key,
        enabled: provider.enabled,
        updated_at: provider.updated_at,
    }
}

fn ai_route_to_dto(route: domain::UserAIRoute) -> UserAIRouteDTO {
    UserAIRouteDTO {
        id: route.id,
        capability: route.capability.as_str().to_string(),
        provider_id: route.provider_id,
        model: route.model,
        embedding_ndims: route.embedding_ndims,
        enabled: route.enabled,
        updated_at: route.updated_at,
    }
}

fn ai_status_to_dto(status: app::user::AICapabilityStatus) -> UserAICapabilityStatusDTO {
    UserAICapabilityStatusDTO {
        capability: status.capability.as_str().to_string(),
        enabled: status.enabled,
        configured: status.configured,
        reason: status.reason,
        provider_id: status.provider_id,
        model: status.model,
    }
}

fn parse_ai_provider_kind(value: &str) -> Result<domain::AIProviderKind, ServerFnError> {
    domain::AIProviderKind::parse(value).ok_or_else(|| ServerFnError::ServerError {
        message: format!("unknown ai provider kind: {value}"),
        code: 400,
        details: None,
    })
}

fn parse_ai_capability(value: &str) -> Result<domain::AICapability, ServerFnError> {
    domain::AICapability::parse(value).ok_or_else(|| ServerFnError::ServerError {
        message: format!("unknown ai capability: {value}"),
        code: 400,
        details: None,
    })
}

fn profile_to_dto(profile: domain::UserProfile) -> UserProfileDTO {
    UserProfileDTO {
        id: profile.id.to_string(),
        display_name: profile.display_name,
        introduction: profile.introduction,
        gender: profile.gender,
        age: profile.age,
    }
}

fn parse_user_id(value: &str) -> Result<domain::UserId, ServerFnError> {
    domain::UserId::parse(value).map_err(|err| ServerFnError::ServerError {
        message: err.to_string(),
        code: 400,
        details: None,
    })
}

fn expectation_to_dto(item: domain::UserHealthExpectation) -> HealthExpectationDTO {
    HealthExpectationDTO {
        id: item.id.to_string(),
        title: item.title,
        summary: item.summary,
        kind: item.kind.as_str().to_string(),
        status: status_to_string(&item.status).to_string(),
        priority: item.priority,
        source: match item.source {
            domain::ExpectationSource::UserExplicit => "user_explicit".to_string(),
            domain::ExpectationSource::AgentProposed { session_id } => {
                format!("agent:{session_id}")
            }
        },
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn companion_to_dto(companion: domain::DiningCompanion) -> DiningCompanionDTO {
    DiningCompanionDTO {
        id: companion.id.to_string(),
        display_name: companion.display_name,
        relationship: companion.relationship,
        introduction: companion.introduction,
        preferred_cuisines: companion
            .diet
            .preferred_cuisines
            .into_iter()
            .map(|item| item.cuisine)
            .collect(),
        avoided_cuisines: companion
            .diet
            .avoided_cuisines
            .into_iter()
            .map(|item| item.cuisine)
            .collect(),
        health_notes: companion.health_notes,
    }
}

fn parse_kind(value: &str) -> domain::HealthExpectationKind {
    match value.trim() {
        "weight_loss" => domain::HealthExpectationKind::WeightLoss,
        "energy_boost" => domain::HealthExpectationKind::EnergyBoost,
        "better_sleep" => domain::HealthExpectationKind::BetterSleep,
        "blood_sugar_control" => domain::HealthExpectationKind::BloodSugarControl,
        other => domain::HealthExpectationKind::Custom(other.to_string()),
    }
}

fn parse_status(value: &str) -> Result<domain::HealthExpectationStatus, ServerFnError> {
    match value.trim() {
        "proposed" => Ok(domain::HealthExpectationStatus::Proposed),
        "active" => Ok(domain::HealthExpectationStatus::Active),
        "archived" => Ok(domain::HealthExpectationStatus::Archived),
        other => Err(ServerFnError::ServerError {
            message: format!("unknown expectation status: {other}"),
            code: 400,
            details: None,
        }),
    }
}

fn status_to_string(value: &domain::HealthExpectationStatus) -> &'static str {
    match value {
        domain::HealthExpectationStatus::Proposed => "proposed",
        domain::HealthExpectationStatus::Active => "active",
        domain::HealthExpectationStatus::Archived => "archived",
    }
}

fn parse_theme(value: &str) -> Option<domain::AppTheme> {
    match value.trim() {
        "light" => Some(domain::AppTheme::Light),
        "dark" => Some(domain::AppTheme::Dark),
        "system" => Some(domain::AppTheme::System),
        _ => None,
    }
}

fn theme_to_string(value: domain::AppTheme) -> String {
    match value {
        domain::AppTheme::Light => "light".to_string(),
        domain::AppTheme::Dark => "dark".to_string(),
        domain::AppTheme::System => "system".to_string(),
    }
}
