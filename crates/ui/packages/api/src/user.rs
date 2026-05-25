use crate::impls::error::api_error;
use crate::impls::state::State;
use dioxus::prelude::*;

const DEMO_USER_ID: &str = "demo-user";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HealthExpectationDto {
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
pub struct UserPreferencesDto {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub preferred_cuisines: Vec<String>,
    pub avoided_cuisines: Vec<String>,
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

#[post("/api/user/preferences/get", state: State)]
pub async fn get_user_preferences() -> Result<UserPreferencesDto, ServerFnError> {
    let user_id = domain::UserId::new_unchecked(DEMO_USER_ID);
    let preferences = state
        .0
        .user
        .preferences_query
        .get_preferences(&user_id)
        .await
        .map_err(api_error)?
        .unwrap_or_else(|| domain::UserPreferences::new(user_id.clone()));

    Ok(UserPreferencesDto {
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

#[post("/api/user/preferences/update", state: State)]
pub async fn update_user_preferences(
    input: UpdatePreferencesInput,
) -> Result<UserPreferencesDto, ServerFnError> {
    let user_id = domain::UserId::new_unchecked(DEMO_USER_ID);
    let command = app::user::UpdateUserPreferencesCommand {
        user_id: user_id.clone(),
        app: domain::AppPreferences {
            theme: input.theme.as_deref().and_then(parse_theme),
            language: input.language.clone().filter(|value| !value.trim().is_empty()),
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

    get_user_preferences().await
}

#[post("/api/user/expectations/list", state: State)]
pub async fn list_health_expectations() -> Result<Vec<HealthExpectationDto>, ServerFnError> {
    let user_id = domain::UserId::new_unchecked(DEMO_USER_ID);
    let expectations = state
        .0
        .user
        .expectation_query
        .list_by_user(&user_id)
        .await
        .map_err(api_error)?;

    Ok(expectations.into_iter().map(expectation_to_dto).collect())
}

#[post("/api/user/expectations/upsert", state: State)]
pub async fn upsert_health_expectation(
    input: UpsertHealthExpectationInput,
) -> Result<Vec<HealthExpectationDto>, ServerFnError> {
    let user_id = domain::UserId::new_unchecked(DEMO_USER_ID);
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

    list_health_expectations().await
}

#[post("/api/user/expectations/confirm", state: State)]
pub async fn confirm_health_expectation(
    expectation_id: String,
) -> Result<Vec<HealthExpectationDto>, ServerFnError> {
    let user_id = domain::UserId::new_unchecked(DEMO_USER_ID);
    state
        .0
        .user
        .expectation_command
        .confirm(app::user::ConfirmHealthExpectationCommand {
            user_id,
            expectation_id: domain::HealthExpectationId::new_unchecked(expectation_id),
        })
        .await
        .map_err(api_error)?;

    list_health_expectations().await
}

#[post("/api/user/expectations/delete", state: State)]
pub async fn delete_health_expectation(
    expectation_id: String,
) -> Result<Vec<HealthExpectationDto>, ServerFnError> {
    let user_id = domain::UserId::new_unchecked(DEMO_USER_ID);
    state
        .0
        .user
        .expectation_command
        .delete(app::user::DeleteHealthExpectationCommand {
            user_id,
            expectation_id: domain::HealthExpectationId::new_unchecked(expectation_id),
        })
        .await
        .map_err(api_error)?;

    list_health_expectations().await
}

fn expectation_to_dto(item: domain::UserHealthExpectation) -> HealthExpectationDto {
    HealthExpectationDto {
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
