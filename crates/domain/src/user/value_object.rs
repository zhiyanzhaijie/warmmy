use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(String);

impl UserId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, UserIdInvalidError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(UserIdInvalidError {
                value: value.to_string(),
            });
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct HealthGoal(String);

impl HealthGoal {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HealthExpectationId(String);

impl HealthExpectationId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, UserIdInvalidError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(UserIdInvalidError {
                value: value.to_string(),
            });
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HealthExpectationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthExpectationKind {
    WeightLoss,
    EnergyBoost,
    BetterSleep,
    BloodSugarControl,
    Custom(String),
}

impl HealthExpectationKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::WeightLoss => "weight_loss",
            Self::EnergyBoost => "energy_boost",
            Self::BetterSleep => "better_sleep",
            Self::BloodSugarControl => "blood_sugar_control",
            Self::Custom(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthExpectationStatus {
    Proposed,
    Active,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectationSource {
    UserExplicit,
    AgentProposed { session_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferenceConfidence {
    Explicit,
    Inferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserIdInvalidError {
    value: String,
}

impl std::fmt::Display for UserIdInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user: invalid user id: '{}'", self.value)
    }
}

impl std::error::Error for UserIdInvalidError {}

impl crate::error::DomainError for UserIdInvalidError {}
