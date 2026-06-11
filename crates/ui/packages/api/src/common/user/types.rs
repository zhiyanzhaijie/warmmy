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
    pub api_key_ref: Option<String>,
    pub has_api_key: bool,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserAIKeyDTO {
    pub id: String,
    pub name: String,
    pub secret_ref: String,
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
    pub api_keys: Vec<UserAIKeyDTO>,
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
    pub api_key_ref: Option<String>,
    pub enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveUserAIKeyInput {
    pub id: Option<String>,
    pub name: String,
    pub api_key: Option<String>,
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
