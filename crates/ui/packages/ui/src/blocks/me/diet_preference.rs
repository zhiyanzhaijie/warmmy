use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, Check, Flame, Pencil, Trash2};

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::hooks::use_IO;
use crate::providers::{set_current_preferences, CurrentUserContext};

use super::common::{merge_tags, BlockMessage, TagListInput};

#[component]
pub fn DietPreferenceSummaryBlock(
    user_id: String,
    on_loaded: EventHandler<user::UserPreferencesDTO>,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut preferred_cuisines = use_signal(Vec::<String>::new);
    let mut avoided_cuisines = use_signal(Vec::<String>::new);
    let mut message = use_signal(String::new);
    let nav = navigator();

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
                Ok(preferences) => {
                    message.set(String::new());
                    preferred_cuisines.set(preferences.preferred_cuisines.clone());
                    avoided_cuisines.set(preferences.avoided_cuisines.clone());
                    set_current_preferences(preferences.clone());
                    on_loaded.call(preferences.clone());
                }
                Err(err) => message.set(format!("加载饮食偏好失败: {err}")),
            }
        }
    });

    let total = preferred_cuisines().len() + avoided_cuisines().len();
    let preview = if total == 0 {
        "还没有设置偏好".to_string()
    } else {
        preferred_cuisines()
            .into_iter()
            .chain(avoided_cuisines())
            .take(3)
            .collect::<Vec<_>>()
            .join("、")
    };

    rsx! {
        Card { class: "min-h-[212px] rounded-2xl border border-border bg-card px-0 py-0 shadow-none",
            CardContent { class: "flex h-full flex-col justify-between gap-5 px-6 py-6",
                div {
                    div { class: "mb-4 flex items-start justify-between gap-3",
                        div { class: "flex h-10 w-10 items-center justify-center rounded-full border border-border bg-card text-foreground shadow-xs",
                            Flame { size: 18 }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-full border border-border px-3 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                            onclick: move |_| {
                                nav.push("/me/preferences");
                            },
                            Pencil { size: 15 }
                        }
                    }
                    p { class: "text-[11px] font-medium uppercase tracking-widest text-muted-foreground", "Preference" }
                    h3 { class: "mt-2 text-xl font-medium tracking-tight text-foreground", "饮食偏好" }
                    p { class: "mt-2 line-clamp-2 text-sm leading-relaxed text-muted-foreground", "{preview}" }
                }
                BlockMessage { message: message() }
                div { class: "flex items-end justify-between gap-3",
                    div {
                        div { class: "text-3xl font-medium tracking-tight text-foreground", "{total}" }
                        div { class: "mt-1 text-xs text-muted-foreground", if loading() { "loading" } else { "偏好与忌口" } }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        class: "rounded-md border border-border px-4 py-2 transition-colors hover:bg-muted text-foreground",
                        onclick: move |_| {
                            nav.push("/me/preferences");
                        },
                        "编辑"
                    }
                }
            }
        }
    }
}

#[component]
pub fn DietPreferenceEditBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let current_user_id = (current_user.user_id)();
    let mut preferences = use_signal(|| None::<user::UserPreferencesDTO>);
    let nav = navigator();

    rsx! {
        div { class: "flex h-full min-h-0 flex-col px-4 py-5 md:px-8 md:py-8",
            div { class: "mx-auto flex h-full min-h-0 w-full max-w-3xl flex-col gap-5",
                div { class: "flex items-center justify-between gap-3",
                    Button { variant: ButtonVariant::Ghost, class: "rounded-full border border-border px-3 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground", onclick: move |_| {
                        nav.push("/me");
                    },
                        ArrowLeft { size: 16 }
                        "返回"
                    }
                    p { class: "text-xs font-medium uppercase tracking-widest text-muted-foreground", "Preferences" }
                }
                div { class: "min-h-0 flex-1 overflow-y-auto pb-28 md:pb-12",
                    DietPreferenceBlock {
                        user_id: current_user_id,
                        on_saved: move |value| preferences.set(Some(value)),
                    }
                }
            }
        }
    }
}

#[component]
pub fn DietPreferenceBlock(
    user_id: String,
    on_saved: EventHandler<user::UserPreferencesDTO>,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut preferred_cuisines = use_signal(Vec::<String>::new);
    let mut preferred_cuisines_input = use_signal(String::new);
    let mut avoided_cuisines = use_signal(Vec::<String>::new);
    let mut avoided_cuisines_input = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut hydrated = use_signal(|| false);

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
                Ok(preferences) if !hydrated() => {
                    message.set(String::new());
                    preferred_cuisines.set(preferences.preferred_cuisines.clone());
                    avoided_cuisines.set(preferences.avoided_cuisines.clone());
                    preferred_cuisines_input.set(String::new());
                    avoided_cuisines_input.set(String::new());
                    set_current_preferences(preferences.clone());
                    on_saved.call(preferences.clone());
                    hydrated.set(true);
                }
                Ok(_) => {}
                Err(err) => message.set(format!("加载饮食偏好失败: {err}")),
            }
        }
    });

    let commit_preferred = move |_| {
        let merged = merge_tags(preferred_cuisines(), &preferred_cuisines_input());
        preferred_cuisines.set(merged);
        preferred_cuisines_input.set(String::new());
    };

    let commit_avoided = move |_| {
        let merged = merge_tags(avoided_cuisines(), &avoided_cuisines_input());
        avoided_cuisines.set(merged);
        avoided_cuisines_input.set(String::new());
    };

    let save_user_id = user_id.clone();
    let save = move |_| {
        let request_user_id = save_user_id.clone();
        async move {
            saving.set(true);
            message.set(String::new());
            let current = match user::get_user_preferences(request_user_id.clone()).await {
                Ok(current) => current,
                Err(err) => {
                    message.set(format!("读取系统偏好失败: {err}"));
                    saving.set(false);
                    return;
                }
            };
            let input = user::UpdatePreferencesInput {
                theme: current.theme,
                language: current.language,
                preferred_cuisines: merge_tags(preferred_cuisines(), &preferred_cuisines_input()),
                avoided_cuisines: merge_tags(avoided_cuisines(), &avoided_cuisines_input()),
            };
            match user::update_user_preferences(request_user_id.clone(), input).await {
                Ok(result) => {
                    preferred_cuisines.set(result.preferred_cuisines.clone());
                    avoided_cuisines.set(result.avoided_cuisines.clone());
                    preferred_cuisines_input.set(String::new());
                    avoided_cuisines_input.set(String::new());
                    set_current_preferences(result.clone());
                    on_saved.call(result);
                    hydrated.set(true);
                    message.set("饮食偏好已保存".to_string());
                }
                Err(err) => message.set(format!("保存饮食偏好失败: {err}")),
            }
            saving.set(false);
        }
    };

    rsx! {
        Card { class: "rounded-2xl border border-border bg-card px-0 py-0 shadow-none",
            CardHeader { class: "gap-2 px-6 pb-0 pt-6",
                div { class: "flex items-center justify-between gap-3",
                    CardTitle { class: "flex items-center gap-2 text-xl font-medium tracking-tight",
                        Flame { size: 18 }
                        "饮食偏好"
                    }
                    span { class: "rounded-full border border-border bg-card px-3 py-1 text-xs text-muted-foreground",
                        "{preferred_cuisines().len() + avoided_cuisines().len()} tags"
                    }
                }
                p { class: "text-sm leading-relaxed text-muted-foreground", "输入多个条目后按回车，保存时会写入长期偏好。" }
            }
            CardContent { class: "space-y-6 px-6 pb-6 pt-5",
                BlockMessage { message: message() }
                TagListInput {
                    label: "偏好的菜系 / 口味",
                    icon: rsx! { Check { size: 16 } },
                    values: preferred_cuisines(),
                    draft: preferred_cuisines_input,
                    placeholder: "云南菜, 粤菜, 清淡",
                    oncommit: commit_preferred,
                    onremove: move |value| preferred_cuisines.write().retain(|item| item != &value),
                }
                TagListInput {
                    label: "忌口 / 避免",
                    icon: rsx! { Trash2 { size: 16 } },
                    values: avoided_cuisines(),
                    draft: avoided_cuisines_input,
                    placeholder: "香菜, 蒜头, 腐乳",
                    oncommit: commit_avoided,
                    onremove: move |value| avoided_cuisines.write().retain(|item| item != &value),
                }
                Button {
                    class: "w-full rounded-md bg-foreground px-5 py-2 text-background shadow-xs transition-opacity hover:opacity-90 sm:w-auto",
                    disabled: saving() || loading(),
                    onclick: save,
                    if saving() { "保存中..." } else { "保存饮食偏好" }
                }
            }
        }
    }
}
