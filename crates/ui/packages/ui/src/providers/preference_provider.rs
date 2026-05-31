use api::user;
use dioxus::prelude::*;

use crate::hooks::use_IO;
use crate::providers::DEFAULT_USER_ID;

#[derive(Clone, Copy)]
pub struct PreferenceContext {
    pub preferences: Signal<Option<user::UserPreferencesDTO>>,
}

#[component]
pub fn PreferenceProvider(children: Element) -> Element {
    let mut preferences = use_signal(|| None);
    use_context_provider(|| PreferenceContext { preferences });

    let loaded = use_IO(move || async move {
        user::get_user_preferences(DEFAULT_USER_ID.to_string()).await
    });

    use_effect(move || {
        if let Some(Ok(next_preferences)) = loaded.read().as_ref() {
            preferences.set(Some(next_preferences.clone()));
            apply_document_theme(&normalize_theme(
                next_preferences.theme.as_deref().unwrap_or("system"),
            ));
        }
    });

    rsx! {
        {children}
    }
}

pub fn set_current_preferences(next_preferences: user::UserPreferencesDTO) {
    if let Some(mut context) = try_consume_context::<PreferenceContext>() {
        context.preferences.set(Some(next_preferences));
    }
}

fn normalize_theme(value: &str) -> &'static str {
    match value.trim() {
        "light" => "light",
        "dark" => "dark",
        _ => "system",
    }
}

fn apply_document_theme(theme: &str) {
    let script = match theme {
        "light" => r#"document.documentElement.setAttribute("data-theme", "light");"#,
        "dark" => r#"document.documentElement.setAttribute("data-theme", "dark");"#,
        _ => r#"document.documentElement.removeAttribute("data-theme");"#,
    };
    document::eval(script);
}
