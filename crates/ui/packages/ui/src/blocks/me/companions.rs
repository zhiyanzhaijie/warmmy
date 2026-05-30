use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, HeartHandshake, Pencil, Save, Trash2, UserPlus, X};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

use super::common::{
    merge_tags, BlockMessage, LabeledInput, LabeledTextarea, MiniTag, TagListInput,
};
use crate::blocks::CurrentUserContext;

#[component]
pub fn CompanionsSummaryBlock(user_id: String) -> Element {
    let mut loading = use_signal(|| false);
    let mut companions = use_signal(Vec::<user::DiningCompanionDTO>::new);
    let mut message = use_signal(String::new);
    let nav = navigator();

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::list_dining_companions(request_user_id).await {
                Ok(items) => companions.set(items),
                Err(err) => message.set(format!("加载关系人失败: {err}")),
            }
            loading.set(false);
        });
    });

    rsx! {
        section { class: "rounded-[2rem] border border-border bg-card px-5 py-5 md:px-6 md:py-6",
            div { class: "mb-5 flex items-start justify-between gap-3",
                div {
                    p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Table circle" }
                    h2 { class: "mt-2 text-2xl font-semibold text-foreground", "常一起吃饭的人" }
                    p { class: "mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground", "首页只展示关系网络摘要，完整新增、编辑和删除进入独立页面。" }
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    class: "rounded-full border border-border px-3",
                    onclick: move |_| {
                        nav.push("/me/companions");
                    },
                    Pencil { size: 15 }
                    "管理"
                }
            }
            BlockMessage { message: message() }
            if loading() {
                div { class: "rounded-[1.5rem] border border-border bg-background/70 px-4 py-6 text-sm text-muted-foreground", "加载中..." }
            } else if companions().is_empty() {
                button {
                    r#type: "button",
                    class: "w-full rounded-[1.5rem] border border-dashed border-border bg-background/70 px-4 py-7 text-left transition hover:bg-muted/50",
                    onclick: move |_| {
                        nav.push("/me/companions");
                    },
                    div { class: "mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-foreground text-background",
                        UserPlus { size: 18 }
                    }
                    div { class: "font-medium text-foreground", "添加家人或朋友" }
                    div { class: "mt-1 text-sm text-muted-foreground", "多人用餐时，agent 会把他们的偏好和健康备注一起考虑。" }
                }
            } else {
                div { class: "grid grid-cols-1 gap-3 md:grid-cols-3",
                    for companion in companions().into_iter().take(3) {
                        CompanionMiniCard { companion }
                    }
                    if companions().len() > 3 {
                        button {
                            r#type: "button",
                            class: "rounded-[1.5rem] border border-border bg-background/70 p-4 text-left transition hover:bg-muted/50",
                            onclick: move |_| {
                                nav.push("/me/companions");
                            },
                            div { class: "text-3xl font-semibold text-foreground", "+{companions().len() - 3}" }
                            div { class: "mt-2 text-sm text-muted-foreground", "查看全部关系人" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CompanionMiniCard(companion: user::DiningCompanionDTO) -> Element {
    let relationship = companion
        .relationship
        .clone()
        .unwrap_or_else(|| "关系人".to_string());
    let first_tag = companion
        .preferred_cuisines
        .first()
        .cloned()
        .or_else(|| companion.health_notes.first().cloned())
        .unwrap_or_else(|| "未设置偏好".to_string());

    rsx! {
        div { class: "rounded-[1.5rem] border border-border bg-background/70 p-4",
            div { class: "flex items-center gap-3",
                div { class: "flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-foreground text-background",
                    HeartHandshake { size: 18 }
                }
                div { class: "min-w-0",
                    div { class: "truncate font-semibold text-foreground", "{companion.display_name}" }
                    div { class: "mt-0.5 text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground", "{relationship}" }
                }
            }
            p { class: "mt-4 line-clamp-2 min-h-10 text-sm leading-relaxed text-muted-foreground", "{companion.introduction}" }
            div { class: "mt-3" }
            MiniTag { label: first_tag }
        }
    }
}

#[component]
pub fn CompanionsBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let user_id = (current_user.user_id)();
    rsx! {
        div { class: "flex h-full min-h-0 flex-col px-4 py-5 md:px-8 md:py-8",
            div { class: "mx-auto flex h-full min-h-0 w-full max-w-5xl flex-col gap-5",
                CompanionsEditor { user_id }
            }
        }
    }
}

#[component]
fn CompanionsEditor(user_id: String) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut companions = use_signal(Vec::<user::DiningCompanionDTO>::new);
    let editing_id = use_signal(|| None::<String>);
    let display_name = use_signal(String::new);
    let relationship = use_signal(String::new);
    let introduction = use_signal(String::new);
    let mut preferred = use_signal(Vec::<String>::new);
    let mut preferred_input = use_signal(String::new);
    let mut avoided = use_signal(Vec::<String>::new);
    let mut avoided_input = use_signal(String::new);
    let mut health_notes = use_signal(Vec::<String>::new);
    let mut health_notes_input = use_signal(String::new);
    let mut message = use_signal(String::new);
    let nav = navigator();

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::list_dining_companions(request_user_id).await {
                Ok(items) => companions.set(items),
                Err(err) => message.set(format!("加载关系人失败: {err}")),
            }
            loading.set(false);
        });
    });

    let start_new = move |_| {
        clear_form(
            editing_id,
            display_name,
            relationship,
            introduction,
            preferred,
            preferred_input,
            avoided,
            avoided_input,
            health_notes,
            health_notes_input,
        );
        message.set(String::new());
        dialog_open.set(true);
    };

    let commit_preferred = move |_| {
        preferred.set(merge_tags(preferred(), &preferred_input()));
        preferred_input.set(String::new());
    };
    let commit_avoided = move |_| {
        avoided.set(merge_tags(avoided(), &avoided_input()));
        avoided_input.set(String::new());
    };
    let commit_health_notes = move |_| {
        health_notes.set(merge_tags(health_notes(), &health_notes_input()));
        health_notes_input.set(String::new());
    };

    let save_user_id = user_id.clone();
    let save = move |_| {
        let request_user_id = save_user_id.clone();
        spawn(async move {
            saving.set(true);
            message.set(String::new());
            let input = user::SaveDiningCompanionInput {
                id: editing_id(),
                display_name: display_name(),
                relationship: Some(relationship()).filter(|value| !value.trim().is_empty()),
                introduction: introduction(),
                preferred_cuisines: merge_tags(preferred(), &preferred_input()),
                avoided_cuisines: merge_tags(avoided(), &avoided_input()),
                health_notes: merge_tags(health_notes(), &health_notes_input()),
            };
            match user::save_dining_companion(request_user_id, input).await {
                Ok(items) => {
                    companions.set(items);
                    clear_form(
                        editing_id,
                        display_name,
                        relationship,
                        introduction,
                        preferred,
                        preferred_input,
                        avoided,
                        avoided_input,
                        health_notes,
                        health_notes_input,
                    );
                    dialog_open.set(false);
                    message.set("关系人已保存".to_string());
                }
                Err(err) => message.set(format!("保存关系人失败: {err}")),
            }
            saving.set(false);
        });
    };

    rsx! {
        div { class: "flex items-center justify-between gap-3",
            Button { variant: ButtonVariant::Ghost, class: "rounded-full border border-border px-3", onclick: move |_| {
                nav.push("/me");
            },
                ArrowLeft { size: 16 }
                "返回"
            }
            p { class: "text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Companions" }
        }
        div { class: "min-h-0 flex-1 overflow-y-auto pb-28 md:pb-12",
            Card { class: "rounded-[2rem] border border-border bg-card px-0 py-0 shadow-none",
                CardHeader { class: "gap-3 px-5 pb-0 pt-5 md:px-6 md:pt-6",
                    div { class: "flex items-start justify-between gap-3",
                        div {
                            CardTitle { class: "flex items-center gap-2 text-xl font-semibold tracking-[-0.3px]",
                                HeartHandshake { size: 18 }
                                "常一起吃饭的人"
                            }
                            p { class: "mt-2 text-sm leading-relaxed text-muted-foreground", "这些人不会切换成当前用户，只会在多人用餐建议时作为关系链约束。" }
                        }
                        Button { size: ButtonSize::IconSm, class: "rounded-full bg-foreground text-background hover:opacity-90", onclick: start_new,
                            UserPlus { size: 16 }
                        }
                    }
                }
                CardContent { class: "space-y-5 px-5 pb-5 pt-5 md:px-6 md:pb-6",
                    BlockMessage { message: message() }

                    div { class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                        if companions().is_empty() && !loading() {
                            div { class: "rounded-[1.5rem] border border-dashed border-border bg-background/70 p-4 text-sm leading-relaxed text-muted-foreground md:col-span-2",
                                "还没有添加家人或朋友。添加后，agent 可以在聚餐、家庭餐建议中一起考虑他们的偏好。"
                            }
                        }
                        for companion in companions() {
                            CompanionCard {
                                key: "{companion.id}",
                                companion: companion.clone(),
                                onedit: move |item| {
                                    apply_companion(
                                        item,
                                        editing_id,
                                        display_name,
                                        relationship,
                                        introduction,
                                        preferred,
                                        preferred_input,
                                        avoided,
                                        avoided_input,
                                        health_notes,
                                        health_notes_input,
                                    );
                                    dialog_open.set(true);
                                },
                                ondelete: {
                                    let request_user_id = user_id.clone();
                                    move |companion_id: String| {
                                        let request_user_id = request_user_id.clone();
                                        spawn(async move {
                                            saving.set(true);
                                            message.set(String::new());
                                            match user::delete_dining_companion(request_user_id, companion_id).await {
                                                Ok(items) => {
                                                    companions.set(items);
                                                    message.set("关系人已删除".to_string());
                                                }
                                                Err(err) => message.set(format!("删除关系人失败: {err}")),
                                            }
                                            saving.set(false);
                                        });
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }

        DialogRoot {
            open: dialog_open(),
            on_open_change: move |open| dialog_open.set(open),
            DialogContent { class: "max-h-[min(86dvh,760px)] w-[calc(100vw-1rem)] max-w-[760px] overflow-hidden rounded-[1.5rem] border border-border bg-card p-0 text-left shadow-2xl sm:w-[calc(100vw-2rem)] sm:rounded-[2rem]",
                div { class: "flex min-h-0 max-h-[min(86dvh,760px)] flex-col",
                    div { class: "flex shrink-0 items-start justify-between gap-4 border-b border-border px-4 py-4 md:px-6",
                        div {
                            DialogTitle {
                                if editing_id().is_some() { "编辑关系人" } else { "新增关系人" }
                            }
                            DialogDescription { "偏好、忌口与健康备注。" }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "rounded-full border border-border",
                            onclick: move |_| dialog_open.set(false),
                            X { size: 16 }
                        }
                    }
                    div { class: "min-h-0 flex-1 space-y-4 overflow-y-auto px-4 py-5 md:px-6",
                        div { class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                            LabeledInput { label: "Name", icon: rsx! { HeartHandshake { size: 16 } }, value: display_name, placeholder: "例如：妈妈、朋友 A" }
                            LabeledInput { label: "Relationship", icon: rsx! { HeartHandshake { size: 16 } }, value: relationship, placeholder: "家人 / 朋友 / 室友" }
                        }
                        LabeledTextarea { label: "Introduction", icon: rsx! { HeartHandshake { size: 16 } }, value: introduction, placeholder: "例如：喜欢清淡，晚餐通常一起吃。" }
                        div { class: "grid grid-cols-1 gap-4 xl:grid-cols-3",
                            TagListInput { label: "偏好", icon: rsx! { HeartHandshake { size: 16 } }, values: preferred(), draft: preferred_input, placeholder: "清淡, 粤菜", oncommit: commit_preferred, onremove: move |value| preferred.write().retain(|item| item != &value) }
                            TagListInput { label: "忌口", icon: rsx! { Trash2 { size: 16 } }, values: avoided(), draft: avoided_input, placeholder: "辣椒, 牛肉", oncommit: commit_avoided, onremove: move |value| avoided.write().retain(|item| item != &value) }
                            TagListInput { label: "健康备注", icon: rsx! { HeartHandshake { size: 16 } }, values: health_notes(), draft: health_notes_input, placeholder: "控糖, 少盐", oncommit: commit_health_notes, onremove: move |value| health_notes.write().retain(|item| item != &value) }
                        }
                    }
                    div { class: "flex shrink-0 items-center justify-between gap-3 border-t border-border px-4 py-4 md:px-6",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-xl border border-border px-4",
                            onclick: move |_| dialog_open.set(false),
                            "取消"
                        }
                        Button { class: "rounded-xl bg-foreground px-5 text-background shadow-sm hover:opacity-90", disabled: saving() || loading(), onclick: save,
                            Save { size: 16 }
                            if saving() { "保存中..." } else { "保存" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CompanionCard(
    companion: user::DiningCompanionDTO,
    onedit: EventHandler<user::DiningCompanionDTO>,
    ondelete: EventHandler<String>,
) -> Element {
    let relationship = companion
        .relationship
        .clone()
        .unwrap_or_else(|| "关系人".to_string());
    let preferred = if companion.preferred_cuisines.is_empty() {
        "无".to_string()
    } else {
        companion.preferred_cuisines.join("、")
    };
    let avoided = if companion.avoided_cuisines.is_empty() {
        "无".to_string()
    } else {
        companion.avoided_cuisines.join("、")
    };
    let edit_companion = companion.clone();
    let delete_companion_id = companion.id.clone();

    rsx! {
        div { class: "rounded-[1.5rem] border border-border bg-background/70 p-4",
            div { class: "flex items-start justify-between gap-3",
                div {
                    div { class: "text-base font-semibold text-foreground", "{companion.display_name}" }
                    div { class: "mt-1 text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground", "{relationship}" }
                }
                div { class: "flex gap-2",
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "rounded-xl border border-border", onclick: move |_| onedit.call(edit_companion.clone()), "编辑" }
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::IconSm, class: "rounded-xl border border-border", onclick: move |_| ondelete.call(delete_companion_id.clone()), Trash2 { size: 15 } }
                }
            }
            if !companion.introduction.trim().is_empty() {
                p { class: "mt-3 text-sm leading-relaxed text-muted-foreground", "{companion.introduction}" }
            }
            div { class: "mt-4 grid gap-2 text-xs text-muted-foreground",
                div { "偏好：{preferred}" }
                div { "忌口：{avoided}" }
                if !companion.health_notes.is_empty() {
                    div { "健康备注：{companion.health_notes.join(\"、\")}" }
                }
            }
        }
    }
}

fn apply_companion(
    companion: user::DiningCompanionDTO,
    mut editing_id: Signal<Option<String>>,
    mut display_name: Signal<String>,
    mut relationship: Signal<String>,
    mut introduction: Signal<String>,
    mut preferred: Signal<Vec<String>>,
    mut preferred_input: Signal<String>,
    mut avoided: Signal<Vec<String>>,
    mut avoided_input: Signal<String>,
    mut health_notes: Signal<Vec<String>>,
    mut health_notes_input: Signal<String>,
) {
    editing_id.set(Some(companion.id));
    display_name.set(companion.display_name);
    relationship.set(companion.relationship.unwrap_or_default());
    introduction.set(companion.introduction);
    preferred.set(companion.preferred_cuisines);
    preferred_input.set(String::new());
    avoided.set(companion.avoided_cuisines);
    avoided_input.set(String::new());
    health_notes.set(companion.health_notes);
    health_notes_input.set(String::new());
}

fn clear_form(
    mut editing_id: Signal<Option<String>>,
    mut display_name: Signal<String>,
    mut relationship: Signal<String>,
    mut introduction: Signal<String>,
    mut preferred: Signal<Vec<String>>,
    mut preferred_input: Signal<String>,
    mut avoided: Signal<Vec<String>>,
    mut avoided_input: Signal<String>,
    mut health_notes: Signal<Vec<String>>,
    mut health_notes_input: Signal<String>,
) {
    editing_id.set(None);
    display_name.set(String::new());
    relationship.set(String::new());
    introduction.set(String::new());
    preferred.set(Vec::new());
    preferred_input.set(String::new());
    avoided.set(Vec::new());
    avoided_input.set(String::new());
    health_notes.set(Vec::new());
    health_notes_input.set(String::new());
}
