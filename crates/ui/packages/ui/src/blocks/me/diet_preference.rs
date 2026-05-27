use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Flame, Trash2};

use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};

use super::common::{merge_tags, BlockMessage, TagListInput};

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

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::get_user_preferences(request_user_id.clone()).await {
                Ok(preferences) => {
                    preferred_cuisines.set(preferences.preferred_cuisines.clone());
                    avoided_cuisines.set(preferences.avoided_cuisines.clone());
                    preferred_cuisines_input.set(String::new());
                    avoided_cuisines_input.set(String::new());
                    on_saved.call(preferences);
                }
                Err(err) => message.set(format!("加载饮食偏好失败: {err}")),
            }
            loading.set(false);
        });
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
        spawn(async move {
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
                    on_saved.call(result);
                    message.set("饮食偏好已保存".to_string());
                }
                Err(err) => message.set(format!("保存饮食偏好失败: {err}")),
            }
            saving.set(false);
        });
    };

    rsx! {
        Card { class: "rounded-[2rem] border border-border bg-card px-0 py-0 shadow-none",
            CardHeader { class: "gap-2 px-5 pb-0 pt-5 md:px-6 md:pt-6",
                CardTitle { class: "flex items-center gap-2 text-xl font-semibold tracking-[-0.3px]",
                    Flame { size: 18 }
                    "饮食偏好"
                }
                p { class: "text-sm leading-relaxed text-muted-foreground", "这些是 agent 做餐食建议时必须读取的长期事实。一次输入多个条目会自动拆成 badge。" }
            }
            CardContent { class: "space-y-4 px-5 pb-5 pt-5 md:px-6 md:pb-6",
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
                    class: "w-full rounded-xl bg-foreground text-background shadow-sm hover:opacity-90 sm:w-auto",
                    disabled: saving() || loading(),
                    onclick: save,
                    if saving() { "保存中..." } else { "保存饮食偏好" }
                }
            }
        }
    }
}
