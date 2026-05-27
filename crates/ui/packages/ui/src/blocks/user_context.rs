use dioxus::prelude::*;

pub const DEFAULT_USER_ID: &str = "default";

#[derive(Clone, Copy)]
pub struct CurrentUserContext {
    pub user_id: Signal<String>,
}

pub fn provide_current_user_context() -> CurrentUserContext {
    let user_id = use_signal(|| DEFAULT_USER_ID.to_string());
    let context = CurrentUserContext { user_id };
    use_context_provider(|| context);
    context
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
