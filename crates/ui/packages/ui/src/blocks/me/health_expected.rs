use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, Check, Flame, Pencil, Plus, X};

use crate::blocks::CurrentUserContext;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

use super::common::{
    BlockMessage, ChoiceOption, LabeledChoiceGroup, LabeledInput, LabeledTextarea,
};

#[component]
pub fn HealthExpectationSummaryBlock(
    user_id: String,
    on_loaded: EventHandler<Vec<user::HealthExpectationDTO>>,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut expectations = use_signal(Vec::<user::HealthExpectationDTO>::new);
    let mut message = use_signal(String::new);
    let nav = navigator();

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::list_health_expectations(request_user_id).await {
                Ok(items) => {
                    on_loaded.call(items.clone());
                    expectations.set(items);
                }
                Err(err) => message.set(format!("加载健康期望失败: {err}")),
            }
            loading.set(false);
        });
    });

    let expectation_items = expectations();
    let active = expectation_items
        .iter()
        .filter(|item| item.status == "active")
        .count();
    let proposed = expectation_items
        .iter()
        .filter(|item| item.status == "proposed")
        .count();
    let preview = expectation_items
        .iter()
        .find(|item| item.status == "active")
        .or_else(|| expectation_items.first())
        .map(|item| item.title.clone())
        .unwrap_or_else(|| "还没有健康期望".to_string());

    rsx! {
        Card { class: "min-h-[212px] rounded-[1.5rem] border border-border bg-card px-0 py-0 shadow-none",
            CardContent { class: "flex h-full flex-col justify-between gap-5 px-5 py-5 md:px-6 md:py-6",
                div {
                    div { class: "mb-4 flex items-start justify-between gap-3",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-full border border-border bg-background text-foreground",
                            Flame { size: 19 }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-full border border-border px-3",
                            onclick: move |_| {
                                nav.push("/me/expectations");
                            },
                            Pencil { size: 15 }
                        }
                    }
                    p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Expectation" }
                    h3 { class: "mt-2 text-xl font-semibold text-foreground", "健康期望" }
                    p { class: "mt-2 line-clamp-2 text-sm leading-relaxed text-muted-foreground", "{preview}" }
                }
                BlockMessage { message: message() }
                div { class: "grid grid-cols-2 gap-2 text-sm",
                    button {
                        r#type: "button",
                        class: "rounded-2xl border border-border bg-background px-3 py-3 text-left transition hover:bg-muted/50",
                        onclick: move |_| {
                            nav.push("/me/expectations");
                        },
                        div { class: "font-doodle text-2xl font-semibold leading-none text-foreground", "{active}" }
                        div { class: "mt-1 text-xs text-muted-foreground", if loading() { "loading" } else { "Active" } }
                    }
                    button {
                        r#type: "button",
                        class: "rounded-2xl border border-border bg-background px-3 py-3 text-left transition hover:bg-muted/50",
                        onclick: move |_| {
                            nav.push("/me/expectations");
                        },
                        div { class: "font-doodle text-2xl font-semibold leading-none text-foreground", "{proposed}" }
                        div { class: "mt-1 text-xs text-muted-foreground", if loading() { "loading" } else { "Proposed" } }
                    }
                }
            }
        }
    }
}

#[component]
pub fn HealthExpectationEditBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let current_user_id = (current_user.user_id)();
    let mut items = use_signal(Vec::<user::HealthExpectationDTO>::new);
    let nav = navigator();

    rsx! {
        div { class: "flex h-full min-h-0 flex-col px-4 py-5 md:px-8 md:py-8",
            div { class: "mx-auto flex h-full min-h-0 w-full max-w-4xl flex-col gap-5",
                div { class: "flex items-center justify-between gap-3",
                    Button { variant: ButtonVariant::Ghost, class: "rounded-full border border-border px-3", onclick: move |_| {
                        nav.push("/me");
                    },
                        ArrowLeft { size: 16 }
                        "返回"
                    }
                    p { class: "text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Expectations" }
                }
                div { class: "min-h-0 flex-1 overflow-y-auto pb-28 md:pb-12",
                    HealthExpectedBlock {
                        user_id: current_user_id,
                        on_loaded: move |value| items.set(value),
                    }
                }
            }
        }
    }
}

#[component]
pub fn HealthExpectedBlock(
    user_id: String,
    on_loaded: EventHandler<Vec<user::HealthExpectationDTO>>,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut expectations = use_signal(Vec::<user::HealthExpectationDTO>::new);
    let mut expectation_id = use_signal(String::new);
    let mut expectation_title = use_signal(String::new);
    let mut expectation_summary = use_signal(String::new);
    let mut expectation_kind = use_signal(|| "weight_loss".to_string());
    let mut expectation_status = use_signal(|| "proposed".to_string());
    let mut expectation_priority = use_signal(|| "50".to_string());
    let mut message = use_signal(String::new);

    use_effect({
        let load_user_id = user_id.clone();
        move || {
            let request_user_id = load_user_id.clone();
            spawn(async move {
                loading.set(true);
                message.set(String::new());
                match user::list_health_expectations(request_user_id.clone()).await {
                    Ok(items) => {
                        on_loaded.call(items.clone());
                        expectations.set(items);
                    }
                    Err(err) => message.set(format!("加载健康期望失败: {err}")),
                }
                loading.set(false);
            });
        }
    });

    let open_new = move |_| {
        reset_expectation_form(
            expectation_id,
            expectation_title,
            expectation_summary,
            expectation_kind,
            expectation_status,
            expectation_priority,
        );
        dialog_open.set(true);
    };

    let save_user_id = user_id.clone();
    let save = move |_| {
        let request_user_id = save_user_id.clone();
        spawn(async move {
            saving.set(true);
            message.set(String::new());
            let priority = expectation_priority().trim().parse::<u8>().unwrap_or(50);
            let input = user::UpsertHealthExpectationInput {
                id: if expectation_id().trim().is_empty() {
                    None
                } else {
                    Some(expectation_id())
                },
                title: expectation_title(),
                summary: expectation_summary(),
                kind: expectation_kind(),
                priority,
                status: expectation_status(),
            };
            match user::upsert_health_expectation(request_user_id.clone(), input).await {
                Ok(items) => {
                    expectations.set(items.clone());
                    on_loaded.call(items);
                    reset_expectation_form(
                        expectation_id,
                        expectation_title,
                        expectation_summary,
                        expectation_kind,
                        expectation_status,
                        expectation_priority,
                    );
                    dialog_open.set(false);
                    message.set("健康期望已保存".to_string());
                }
                Err(err) => message.set(format!("保存健康期望失败: {err}")),
            }
            saving.set(false);
        });
    };

    rsx! {
        Card { class: "rounded-[2rem] border border-border bg-card px-0 py-0 shadow-none xl:sticky xl:top-8 xl:self-start",
            CardHeader { class: "gap-3 px-5 pb-0 pt-5 md:px-6 md:pt-6",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Expectation stack" }
                        CardTitle { class: "mt-2 flex items-center gap-2 text-2xl font-semibold",
                            Flame { size: 18 }
                            "健康期望"
                        }
                        p { class: "mt-2 text-sm leading-relaxed text-muted-foreground", "阶段性目标用小卡片承载。点击卡片编辑，新增只占一个按钮。" }
                    }
                    div { class: "flex shrink-0 gap-2",
                        Button {
                            size: ButtonSize::IconSm,
                            class: "rounded-full bg-foreground text-background hover:opacity-90",
                            onclick: open_new,
                            Plus { size: 16 }
                        }
                    }
                }
            }
            CardContent { class: "space-y-3 px-5 pb-5 pt-5 md:px-6 md:pb-6",
                BlockMessage { message: message() }
                if loading() {
                    div { class: "rounded-2xl border border-border bg-background/70 px-4 py-6 text-sm text-muted-foreground", "加载中..." }
                } else if expectations().is_empty() {
                    EmptyExpectationCard { on_new: open_new }
                } else {
                    div { class: "grid grid-cols-1 gap-3",
                        for item in expectations() {
                            HealthExpectationMiniCard {
                                key: "{item.id}",
                                item: item.clone(),
                                on_edit: move |current: user::HealthExpectationDTO| {
                                    expectation_id.set(current.id.clone());
                                    expectation_title.set(current.title.clone());
                                    expectation_summary.set(current.summary.clone());
                                    expectation_kind.set(current.kind.clone());
                                    expectation_status.set(current.status.clone());
                                    expectation_priority.set(current.priority.to_string());
                                    dialog_open.set(true);
                                },
                                on_confirm: {
                                    let action_user_id = user_id.clone();
                                    move |id| {
                                    let request_user_id = action_user_id.clone();
                                    spawn(async move {
                                        match user::confirm_health_expectation(request_user_id.clone(), id).await {
                                            Ok(items) => {
                                                expectations.set(items.clone());
                                                on_loaded.call(items);
                                                message.set("已确认健康期望".to_string());
                                            }
                                            Err(err) => message.set(format!("确认失败: {err}")),
                                        }
                                    });
                                }
                                },
                                on_delete: {
                                    let action_user_id = user_id.clone();
                                    move |id| {
                                    let request_user_id = action_user_id.clone();
                                    spawn(async move {
                                        match user::delete_health_expectation(request_user_id.clone(), id).await {
                                            Ok(items) => {
                                                expectations.set(items.clone());
                                                on_loaded.call(items);
                                                message.set("已删除健康期望".to_string());
                                            }
                                            Err(err) => message.set(format!("删除失败: {err}")),
                                        }
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
            DialogContent { class: "max-h-[min(86dvh,760px)] w-[calc(100vw-1rem)] max-w-[680px] overflow-hidden rounded-[1.5rem] border border-border bg-card p-0 text-left shadow-2xl sm:w-[calc(100vw-2rem)] sm:rounded-[2rem]",
                div { class: "flex min-h-0 max-h-[min(86dvh,760px)] flex-col",
                    div { class: "flex shrink-0 items-start justify-between gap-4 border-b border-border px-4 py-4 md:px-6",
                        div {
                            DialogTitle {
                                if expectation_id().trim().is_empty() { "新增健康期望" } else { "编辑健康期望" }
                            }
                            DialogDescription { "阶段目标与优先级。" }
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
                        LabeledInput {
                            label: "Title",
                            icon: rsx! { Flame { size: 16 } },
                            value: expectation_title,
                            placeholder: "减脂 / 提神 / 控糖",
                        }
                        LabeledTextarea {
                            label: "Summary",
                            icon: rsx! { Check { size: 16 } },
                            value: expectation_summary,
                            placeholder: "描述期望，例如：未来两周晚餐少油少糖，优先高蛋白。",
                        }
                        div { class: "grid grid-cols-1 gap-3 md:grid-cols-3",
                            LabeledChoiceGroup {
                                label: "Kind",
                                icon: rsx! { Flame { size: 16 } },
                                value: expectation_kind,
                                options: vec![
                                    ChoiceOption::new("weight_loss", "减脂"),
                                    ChoiceOption::new("energy_boost", "提神"),
                                    ChoiceOption::new("better_sleep", "睡眠"),
                                    ChoiceOption::new("blood_sugar_control", "控糖"),
                                ],
                            }
                            LabeledChoiceGroup {
                                label: "Status",
                                icon: rsx! { Check { size: 16 } },
                                value: expectation_status,
                                options: vec![
                                    ChoiceOption::new("proposed", "Proposed"),
                                    ChoiceOption::new("active", "Active"),
                                    ChoiceOption::new("archived", "Archived"),
                                ],
                            }
                            LabeledInput {
                                label: "Priority",
                                icon: rsx! { Plus { size: 16 } },
                                value: expectation_priority,
                                placeholder: "50",
                            }
                        }
                    }
                    div { class: "flex shrink-0 items-center justify-between gap-3 border-t border-border px-4 py-4 md:px-6",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-xl border border-border px-4",
                            onclick: move |_| dialog_open.set(false),
                            "取消"
                        }
                        Button {
                            class: "rounded-xl bg-foreground px-5 text-background shadow-sm hover:opacity-90",
                            disabled: saving(),
                            onclick: save,
                            if saving() { "保存中..." } else { "保存" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyExpectationCard(on_new: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "w-full rounded-[1.5rem] border border-dashed border-border bg-background/70 px-4 py-8 text-left text-sm leading-relaxed text-muted-foreground transition hover:bg-muted/50",
            onclick: move |event| on_new.call(event),
            div { class: "mb-2 flex h-10 w-10 items-center justify-center rounded-full bg-foreground text-background",
                Plus { size: 18 }
            }
            div { class: "font-medium text-foreground", "添加第一个健康期望" }
            div { class: "mt-1", "例如减脂、控糖、改善睡眠或短期提神。" }
        }
    }
}

#[component]
fn HealthExpectationMiniCard(
    item: user::HealthExpectationDTO,
    on_edit: EventHandler<user::HealthExpectationDTO>,
    on_confirm: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    let status_badge = match item.status.as_str() {
        "active" => "bg-foreground text-background",
        "archived" => "bg-muted text-muted-foreground",
        _ => "bg-secondary text-secondary-foreground",
    };
    let is_active = item.status == "active";
    let expectation_id = item.id.clone();
    let expectation_id_for_delete = expectation_id.clone();
    let item_for_edit = item.clone();

    rsx! {
        div { class: "group rounded-[1.45rem] border border-border bg-background/80 p-4 transition hover:bg-background",
            button { r#type: "button", class: "w-full text-left", onclick: move |_| on_edit.call(item_for_edit.clone()),
                div { class: "flex items-start justify-between gap-3",
                    div { class: "min-w-0",
                        h3 { class: "line-clamp-1 text-base font-semibold tracking-[-0.2px] text-foreground", "{item.title}" }
                        p { class: "mt-1 line-clamp-2 text-sm leading-relaxed text-muted-foreground", "{item.summary}" }
                    }
                    span { class: format!("shrink-0 rounded-full px-3 py-1 text-xs font-semibold {}", status_badge), "{item.status}" }
                }
                div { class: "mt-3 flex flex-wrap gap-2 text-xs text-muted-foreground",
                    span { class: "rounded-full border border-border bg-card px-2 py-1", "{item.kind}" }
                    span { class: "rounded-full border border-border bg-card px-2 py-1", "priority {item.priority}" }
                    span { class: "rounded-full border border-border bg-card px-2 py-1", "{item.source}" }
                }
            }
            div { class: "mt-3 flex gap-2",
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "flex-1 rounded-xl border border-border",
                    onclick: move |_| on_confirm.call(expectation_id.clone()),
                    disabled: is_active,
                    "Confirm"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "flex-1 rounded-xl border border-destructive/30 text-destructive",
                    onclick: move |_| on_delete.call(expectation_id_for_delete.clone()),
                    "Delete"
                }
            }
        }
    }
}

fn reset_expectation_form(
    mut expectation_id: Signal<String>,
    mut expectation_title: Signal<String>,
    mut expectation_summary: Signal<String>,
    mut expectation_kind: Signal<String>,
    mut expectation_status: Signal<String>,
    mut expectation_priority: Signal<String>,
) {
    expectation_id.set(String::new());
    expectation_title.set(String::new());
    expectation_summary.set(String::new());
    expectation_kind.set("weight_loss".to_string());
    expectation_status.set("proposed".to_string());
    expectation_priority.set("50".to_string());
}
