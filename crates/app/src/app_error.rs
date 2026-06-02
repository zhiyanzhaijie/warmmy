use thiserror::Error;

use domain::error::DomainError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Upstream error: {0}")]
    Upstream(String),
    #[error("Domain error: {0}")]
    Domain(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(message) if message == "AI capability is not configured: chat" => {
                "ai.chat_not_configured"
            }
            Self::Validation(message) if message == "AI capability is not configured: vision" => {
                "ai.vision_not_configured"
            }
            Self::Validation(message) if message.starts_with("AI capability is not configured: ") => {
                "ai.capability_not_configured"
            }
            Self::Validation(message) if message.starts_with("unsupported image mime type: ") => {
                "media.image_unsupported"
            }
            Self::Validation(message) if message == "empty conversation input" => {
                "chat.empty_input"
            }
            Self::Validation(_) => "validation.invalid_input",
            Self::Database(_) => "memory.database_unavailable",
            Self::Upstream(_) => "model.upstream_unavailable",
            Self::Domain(_) => "domain.invalid_state",
            Self::NotFound(message)
                if message.starts_with("ephemeral image not found: ")
                    || message.starts_with("ephemeral image expired: ") =>
            {
                "media.image_unavailable"
            }
            Self::NotFound(_) => "resource.not_found",
            Self::Internal(_) => "internal.error",
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::Database(_) => "database",
            Self::Upstream(_) => "upstream",
            Self::Domain(_) => "domain",
            Self::NotFound(_) => "not_found",
            Self::Internal(_) => "internal",
        }
    }

    pub fn validation(err: impl ToString) -> Self {
        Self::Validation(err.to_string())
    }

    pub fn database(err: impl ToString) -> Self {
        Self::Database(err.to_string())
    }

    pub fn upstream(err: impl ToString) -> Self {
        Self::Upstream(err.to_string())
    }

    pub fn internal(err: impl ToString) -> Self {
        Self::Internal(err.to_string())
    }
}

impl<E> From<E> for AppError
where
    E: DomainError,
{
    fn from(err: E) -> Self {
        Self::Domain(err.to_string())
    }
}
