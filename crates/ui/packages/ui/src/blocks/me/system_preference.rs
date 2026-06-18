use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Globe, Palette, Sparkles};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::sheet::{
    Sheet, SheetContent, SheetContentClose, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle,
};
use crate::hooks::use_IO;
use crate::providers::{set_current_preferences, PreferenceContext};

use super::common::{
    apply_document_theme, normalize_theme, BlockMessage, ChoiceOption, LabeledChoiceGroup,
    LabeledInput,
};

#[component]
pub fn SystemPreferenceBlock(
    user_id: String,
    on_saved: EventHandler<user::UserPreferencesDTO>,
) -> Element {
    let preference_context = use_context::<PreferenceContext>();
    let initial_preferences = (preference_context.preferences)();
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut sheet_open = use_signal(|| false);
    let mut theme = use_signal({
        let initial_preferences = initial_preferences.clone();
        move || {
            initial_preferences
                .as_ref()
                .and_then(|preferences| preferences.theme.as_deref())
                .map(normalize_theme)
                .unwrap_or_else(|| "system".to_string())
        }
    });
    let mut language = use_signal({
        let initial_preferences = initial_preferences.clone();
        move || {
            initial_preferences
                .as_ref()
                .and_then(|preferences| preferences.language.as_deref())
                .unwrap_or("zh-CN")
                .to_string()
        }
    });
    let mut message = use_signal(String::new);
    let mut initialized = use_signal({
        let has_initial_preferences = initial_preferences.is_some();
        move || has_initial_preferences
    });

    let loaded_preferences = use_IO({
        let user_id = user_id.clone();
        move || {
            let request_user_id = user_id.clone();
            async move { user::get_user_preferences(request_user_id).await }
        }
    });
    use_effect(move || {
        loading.set(loaded_preferences.read().is_none());
        if let Some(result) = loaded_preferences.read().as_ref() {
            match result {
                Ok(preferences) if !initialized() => {
                    message.set(String::new());
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
                    set_current_preferences(preferences.clone());
                    on_saved.call(preferences.clone());
                    initialized.set(true);
                }
                Ok(_) => {}
                Err(err) => message.set(format!("加载系统偏好失败: {err}")),
            }
        }
    });

    let save_user_id = user_id.clone();
    let save = move |_| {
        let request_user_id = save_user_id.clone();
        async move {
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
                    set_current_preferences(result.clone());
                    on_saved.call(result);
                    initialized.set(true);
                    message.set("系统偏好已保存".to_string());
                    sheet_open.set(false);
                }
                Err(err) => message.set(format!("保存系统偏好失败: {err}")),
            }
            saving.set(false);
        }
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
            class: "h-12 w-12 rounded-full border border-border bg-background text-muted-foreground shadow-none transition-colors hover:border-foreground/40 hover:bg-foreground/5 hover:text-foreground",
            aria_label: "打开系统偏好",
            onclick: move |_| sheet_open.set(true),
            Palette { size: 22 }
        }

        Sheet {
            open: sheet_open(),
            on_open_change: move |open| sheet_open.set(open),
            SheetContent {
                side: SheetSide::Right,
                class: "w-[min(88vw,26rem)] max-w-none gap-0 border-l border-border bg-background p-0 text-foreground shadow-2xl",
                div { class: "flex h-full min-h-0 flex-col overflow-hidden",
                    SheetHeader { class: "relative shrink-0 border-b border-border/50 px-6 pb-5 pt-6",
                        div { class: "relative pr-10",
                            p { class: "text-[11px] font-medium uppercase tracking-widest text-muted-foreground", "System" }
                            div { class: "mt-3 flex items-center gap-3",
                                div { class: "grid h-10 w-10 place-items-center rounded-xl border border-border bg-background text-foreground shadow-xs" ,
                                    Palette { size: 18 }
                                }
                                div { class: "min-w-0",
                                    SheetTitle { class: "text-lg font-medium tracking-tight", "系统偏好" }
                                    SheetDescription { class: "mt-1", "把界面调成你最顺眼的样子吧。" }
                                }
                            }
                        }
                        SheetContentClose { class: "right-5 top-6 rounded-full border border-border text-muted-foreground transition-colors hover:border-foreground/40 hover:bg-foreground/5 hover:text-foreground" }
                    }

                    div { class: "min-h-0 flex-1 overflow-y-auto px-6 py-6",
                        BlockMessage { message: message() }

                        div { class: "mb-6 grid grid-cols-2 gap-3",
                            div { class: "rounded-xl border border-border bg-background px-4 py-4",
                                div { class: "flex items-center gap-2 text-[11px] font-medium uppercase tracking-widest text-muted-foreground",
                                    Palette { size: 14 }
                                    "Theme"
                                }
                                div { class: "mt-2 text-[20px] font-normal leading-[1.25] text-foreground", "{theme_label}" }
                            }
                            div { class: "rounded-xl border border-border bg-background px-4 py-4",
                                div { class: "flex items-center gap-2 text-[11px] font-medium uppercase tracking-widest text-muted-foreground",
                                    Globe { size: 14 }
                                    "Lang"
                                }
                                div { class: "mt-2 truncate text-[20px] font-normal leading-[1.25] text-foreground", "{language_label}" }
                            }
                        }

                        div { class: "space-y-6",
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

                        div { class: "mt-8 rounded-xl border border-dashed border-border bg-background px-4 py-4",
                            div { class: "flex items-start gap-3",
                                Sparkles { class: "mt-0.5 text-muted-foreground", size: 16 }
                                p { class: "text-xs leading-relaxed text-muted-foreground",
                                    "挑一个你喜欢的主题和语言，屋米会记住你的习惯。"
                                }
                            }
                        }
                    }

                    SheetFooter { class: "shrink-0 border-t border-border/50 bg-background px-6 py-4",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-[6px] border border-border bg-transparent px-5 py-2 text-muted-foreground transition-all hover:border-foreground/40 hover:bg-foreground/5 hover:text-foreground",
                            onclick: move |_| sheet_open.set(false),
                            "取消"
                        }
                        Button {
                            class: "rounded-[6px] bg-foreground px-5 py-2 text-background shadow-[inset_0_0.5px_0_rgba(255,255,255,0.2),inset_0_0_0_0.5px_rgba(0,0,0,0.2),0_1px_2px_rgba(0,0,0,0.05)] transition-all hover:opacity-80 focus:shadow-[0_4px_12px_rgba(0,0,0,0.1)]",
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
