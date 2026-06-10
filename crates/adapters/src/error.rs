use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("integration is not configured: {0}")]
    NotConfigured(String),
}
