use api::user;
use dioxus::prelude::*;

use crate::hooks::use_IO;

pub const DEFAULT_USER_ID: &str = "default";

#[derive(Clone, Copy)]
pub struct CurrentUserContext {
    pub user_id: Signal<String>,
}

#[component]
pub fn UserProvider(children: Element) -> Element {
    let user_id = use_signal(|| DEFAULT_USER_ID.to_string());
    use_context_provider(|| CurrentUserContext { user_id });

    use_IO(move || async move {
        let _ = user::get_user_profile(DEFAULT_USER_ID.to_string()).await;
    });

    rsx! {
        {children}
    }
}

pub fn current_user_id() -> String {
    try_consume_context::<CurrentUserContext>()
        .map(|context| (context.user_id)())
        .unwrap_or_else(|| DEFAULT_USER_ID.to_string())
}

pub fn set_current_user_id(user_id: String) {
    if let Some(mut context) = try_consume_context::<CurrentUserContext>() {
        context.user_id.set(user_id);
    }
}
