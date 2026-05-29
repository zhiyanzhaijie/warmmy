use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Globe, Palette, Pencil, X};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

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
    let mut dialog_open = use_signal(|| false);
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

    let language_label = if language().trim().is_empty() {
        "未设置".to_string()
    } else {
        language()
    };
    let theme_label = match theme().as_str() {
        "light" => "Light",
        "dark" => "Dark",
        _ => "System",
    };

    rsx! {
        section { class: "flex min-h-0 border-t border-border bg-background/45 lg:h-full lg:border-l lg:border-t-0",
            div { class: "flex min-h-0 flex-1 flex-col justify-between gap-4 px-5 py-5 md:px-6 lg:py-8",
                div { class: "space-y-4",
                    div { class: "flex items-start justify-between gap-3",
                        div { class: "min-w-0",
                            p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "System" }
                            h3 { class: "mt-2 text-lg font-semibold text-foreground", "系统偏好" }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "rounded-full border border-border",
                            onclick: move |_| dialog_open.set(true),
                            Pencil { size: 15 }
                        }
                    }
                    BlockMessage { message: message() }
                }
                div { class: "grid grid-cols-2 gap-2 text-sm lg:grid-cols-1",
                    div { class: "rounded-[1.25rem] border border-border bg-card/80 px-3 py-3",
                        div { class: "flex items-center gap-2 text-[11px] text-muted-foreground",
                            Palette { size: 14 }
                            "Theme"
                        }
                        div { class: "mt-1 text-base font-semibold text-foreground", "{theme_label}" }
                    }
                    div { class: "rounded-[1.25rem] border border-border bg-card/80 px-3 py-3",
                        div { class: "flex items-center gap-2 text-[11px] text-muted-foreground",
                            Globe { size: 14 }
                            "Language"
                        }
                        div { class: "mt-1 text-base font-semibold text-foreground", "{language_label}" }
                    }
                }
            }
        }

        DialogRoot {
            open: dialog_open(),
            on_open_change: move |open| dialog_open.set(open),
            DialogContent { class: "max-h-[min(86dvh,680px)] w-[calc(100vw-1rem)] max-w-[560px] overflow-hidden rounded-[1.5rem] border border-border bg-card p-0 text-left shadow-2xl sm:w-[calc(100vw-2rem)] sm:rounded-[2rem]",
                div { class: "flex min-h-0 max-h-[min(86dvh,680px)] flex-col",
                    div { class: "flex shrink-0 items-start justify-between gap-4 border-b border-border px-4 py-4 md:px-6",
                        div {
                            DialogTitle { "编辑系统偏好" }
                            DialogDescription { "影响界面主题与默认语言。" }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "rounded-full border border-border",
                            onclick: move |_| dialog_open.set(false),
                            X { size: 16 }
                        }
                    }
                    div { class: "min-h-0 flex-1 space-y-5 overflow-y-auto px-4 py-5 md:px-6",
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
                    }
                    div { class: "flex shrink-0 flex-col gap-2 border-t border-border px-4 py-4 sm:flex-row sm:justify-end md:px-6",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-xl border border-border px-4",
                            onclick: move |_| dialog_open.set(false),
                            "取消"
                        }
                Button {
                            class: "rounded-xl bg-foreground px-5 text-background shadow-sm hover:opacity-90",
                    disabled: saving() || loading(),
                    onclick: save,
                    if saving() { "保存中..." } else { "保存系统偏好" }
                }
                    }
                }
            }
        }
    }
}
