use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Globe, Palette};

use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};

use super::common::{
    apply_document_theme, normalize_theme, BlockMessage, ChoiceOption, LabeledChoiceGroup,
    LabeledInput,
};

#[component]
pub fn SystemPreferenceBlock(
    user_id: String,
    on_saved: EventHandler<user::UserPreferencesDTO>,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut theme = use_signal(|| "system".to_string());
    let mut language = use_signal(|| "zh-CN".to_string());
    let mut message = use_signal(String::new);

    use_effect(move || {
        apply_document_theme(&theme());
    });

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::get_user_preferences(request_user_id.clone()).await {
                Ok(preferences) => {
                    let next_theme = normalize_theme(
                        preferences
                            .theme
                            .clone()
                            .unwrap_or_else(|| "system".to_string())
                            .as_str(),
                    );
                    theme.set(next_theme.clone());
                    apply_document_theme(&next_theme);
                    language.set(
                        preferences
                            .language
                            .clone()
                            .unwrap_or_else(|| "zh-CN".to_string()),
                    );
                    on_saved.call(preferences);
                }
                Err(err) => message.set(format!("加载系统偏好失败: {err}")),
            }
            loading.set(false);
        });
    });

    let save_user_id = user_id.clone();
    let save = move |_| {
        let request_user_id = save_user_id.clone();
        spawn(async move {
            saving.set(true);
            message.set(String::new());
            let current = match user::get_user_preferences(request_user_id.clone()).await {
                Ok(current) => current,
                Err(err) => {
                    message.set(format!("读取饮食偏好失败: {err}"));
                    saving.set(false);
                    return;
                }
            };
            let input = user::UpdatePreferencesInput {
                theme: Some(theme()),
                language: Some(language()),
                preferred_cuisines: current.preferred_cuisines,
                avoided_cuisines: current.avoided_cuisines,
            };
            match user::update_user_preferences(request_user_id.clone(), input).await {
                Ok(result) => {
                    let next_theme = normalize_theme(
                        result
                            .theme
                            .clone()
                            .unwrap_or_else(|| "system".to_string())
                            .as_str(),
                    );
                    theme.set(next_theme.clone());
                    apply_document_theme(&next_theme);
                    language.set(
                        result
                            .language
                            .clone()
                            .unwrap_or_else(|| "zh-CN".to_string()),
                    );
                    on_saved.call(result);
                    message.set("系统偏好已保存".to_string());
                }
                Err(err) => message.set(format!("保存系统偏好失败: {err}")),
            }
            saving.set(false);
        });
    };

    rsx! {
        Card { class: "rounded-[2rem] border border-border bg-card px-0 py-0 shadow-none",
            CardHeader { class: "gap-2 px-5 pb-0 pt-5 md:px-6 md:pt-6",
                CardTitle { class: "flex items-center gap-2 text-xl font-semibold tracking-[-0.3px]",
                    Palette { size: 18 }
                    "系统偏好"
                }
                p { class: "text-sm leading-relaxed text-muted-foreground", "只保留会影响产品体验的设置，不和饮食画像混在一起。" }
            }
            CardContent { class: "space-y-5 px-5 pb-5 pt-5 md:px-6 md:pb-6",
                BlockMessage { message: message() }
                LabeledChoiceGroup {
                    label: "Theme",
                    icon: rsx! { Palette { size: 16 } },
                    value: theme,
                    onselect: move |value: String| {
                        let next = normalize_theme(&value);
                        theme.set(next.clone());
                        apply_document_theme(&next);
                    },
                    options: vec![
                        ChoiceOption::new("system", "System"),
                        ChoiceOption::new("light", "Light"),
                        ChoiceOption::new("dark", "Dark"),
                    ],
                }
                LabeledInput {
                    label: "Language",
                    icon: rsx! { Globe { size: 16 } },
                    value: language,
                    placeholder: "zh-CN",
                }
                Button {
                    class: "w-full rounded-xl bg-foreground text-background shadow-sm hover:opacity-90 sm:w-auto",
                    disabled: saving() || loading(),
                    onclick: save,
                    if saving() { "保存中..." } else { "保存系统偏好" }
                }
            }
        }
    }
}
