use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("adapter is not configured: {0}")]
    NotConfigured(String),
}
