/// Marker trait for domain-layer errors.
///
/// Upper layers can map `DomainError` into their own error model
/// without depending on concrete domain error types.
pub trait DomainError: std::error::Error + Send + Sync + 'static {}
