use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct LocalAppState(pub Arc<infra::setup::AppState>);

static FALLBACK_STATE: OnceCell<LocalAppState> = OnceCell::const_new();

pub fn use_local_app_state_provider(state: LocalAppState) -> LocalAppState {
    use_context_provider(|| state)
}

pub async fn init() -> Result<LocalAppState, ServerFnError> {
    let container = infra::setup::init_app_container()
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(LocalAppState(Arc::new(container)))
}

pub async fn state() -> Result<LocalAppState, ServerFnError> {
    if let Some(state) = try_consume_context::<LocalAppState>() {
        return Ok(state);
    }

    FALLBACK_STATE.get_or_try_init(init).await.cloned()
}
