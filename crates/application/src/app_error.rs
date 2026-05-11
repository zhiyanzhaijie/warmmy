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
