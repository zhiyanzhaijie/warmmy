use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayCycle(String);

impl DayCycle {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, DayCycleInvalidError> {
        let value = value.trim().to_lowercase();
        match value.as_str() {
            "breakfast" | "lunch" | "dinner" | "snack" => Ok(Self(value)),
            _ => Err(DayCycleInvalidError { value }),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DayCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayCycleInvalidError {
    value: String,
}

impl std::fmt::Display for DayCycleInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "meal: invalid day cycle: '{}', expected one of breakfast|lunch|dinner|snack",
            self.value
        )
    }
}

impl std::error::Error for DayCycleInvalidError {}

impl crate::error::DomainError for DayCycleInvalidError {}
