use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Globe, Palette, Sparkles};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::sheet::{
    Sheet, SheetContent, SheetContentClose, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle,
};

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
    let mut sheet_open = use_signal(|| false);
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
                    sheet_open.set(false);
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
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconLg,
            class: "h-12 w-12 rounded-full border border-border bg-card/80 text-foreground shadow-sm backdrop-blur transition hover:bg-background/90",
            aria_label: "打开系统偏好",
            onclick: move |_| sheet_open.set(true),
            Palette { size: 22 }
        }

        Sheet {
            open: sheet_open(),
            on_open_change: move |open| sheet_open.set(open),
            SheetContent {
                side: SheetSide::Right,
                class: "w-[min(88vw,26rem)] max-w-none gap-0 border-l border-border bg-card p-0 text-foreground shadow-2xl",
                div { class: "flex h-full min-h-0 flex-col overflow-hidden",
                    SheetHeader { class: "relative shrink-0 border-b border-border px-5 pb-5 pt-6 md:px-6",
                        div { class: "pointer-events-none absolute -right-16 -top-16 h-44 w-44 rounded-full bg-primary/10 blur-3xl" }
                        div { class: "relative pr-10",
                            p { class: "text-[11px] font-semibold uppercase tracking-[0.24em] text-muted-foreground", "System" }
                            div { class: "mt-3 flex items-center gap-3",
                                div { class: "grid h-11 w-11 place-items-center rounded-2xl border border-border bg-background text-foreground" ,
                                    Palette { size: 21 }
                                }
                                div { class: "min-w-0",
                                    SheetTitle { "系统偏好" }
                                    SheetDescription { "调整界面主题与默认语言。" }
                                }
                            }
                        }
                        SheetContentClose { class: "right-5 top-6 rounded-full border border-border text-muted-foreground transition hover:bg-background hover:text-foreground" }
                    }

                    div { class: "min-h-0 flex-1 overflow-y-auto px-5 py-5 md:px-6",
                        BlockMessage { message: message() }

                        div { class: "mb-6 grid grid-cols-2 gap-3",
                            div { class: "rounded-[1.4rem] border border-border bg-background/70 px-4 py-4",
                                div { class: "flex items-center gap-2 text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground",
                                    Palette { size: 14 }
                                    "Theme"
                                }
                                div { class: "mt-2 text-xl font-semibold text-foreground", "{theme_label}" }
                            }
                            div { class: "rounded-[1.4rem] border border-border bg-background/70 px-4 py-4",
                                div { class: "flex items-center gap-2 text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground",
                                    Globe { size: 14 }
                                    "Lang"
                                }
                                div { class: "mt-2 truncate text-xl font-semibold text-foreground", "{language_label}" }
                            }
                        }

                        div { class: "space-y-5",
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

                        div { class: "mt-7 rounded-[1.6rem] border border-dashed border-border bg-background/45 px-4 py-4",
                            div { class: "flex items-start gap-3",
                                Sparkles { class: "mt-0.5 text-muted-foreground", size: 17 }
                                p { class: "text-xs leading-relaxed text-muted-foreground",
                                    "这些设置会立即影响本机界面表现，保存后同步写入当前用户偏好。"
                                }
                            }
                        }
                    }

                    SheetFooter { class: "shrink-0 border-t border-border bg-card px-5 py-4 md:px-6",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-xl border border-border px-4",
                            onclick: move |_| sheet_open.set(false),
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
