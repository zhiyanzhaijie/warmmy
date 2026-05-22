use dioxus::prelude::*;

use crate::impls::state::State;

#[get("/api/server/ready", state: State)]
pub async fn server_ready() -> ServerFnResult<bool> {
    Ok(state.0.nutrition.repo.is_ready().await)
}
