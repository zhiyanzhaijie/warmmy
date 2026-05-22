#[cfg(feature = "server")]
pub type State = dioxus::server::axum::extract::Extension<std::sync::Arc<infra::setup::AppState>>;

#[cfg(not(feature = "server"))]
#[allow(dead_code)]
pub struct State;
