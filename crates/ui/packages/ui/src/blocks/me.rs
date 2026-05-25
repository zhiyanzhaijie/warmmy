use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Flame, Globe, Palette, Plus, RefreshCw, Trash2};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::input::Input;

#[component]
pub fn MeBlock() -> Element {
    let mut loading = use_signal(|| false);
    let mut saving_preferences = use_signal(|| false);
    let mut saving_expectation = use_signal(|| false);
    let mut theme = use_signal(|| "system".to_string());
    let mut language = use_signal(|| "zh-CN".to_string());
    let mut preferred_cuisines = use_signal(Vec::<String>::new);
    let mut preferred_cuisines_input = use_signal(String::new);
    let mut avoided_cuisines = use_signal(Vec::<String>::new);
    let mut avoided_cuisines_input = use_signal(String::new);
    let mut expectations = use_signal(Vec::<user::HealthExpectationDto>::new);
    let mut expectation_id = use_signal(String::new);
    let mut expectation_title = use_signal(String::new);
    let mut expectation_summary = use_signal(String::new);
    let mut expectation_kind = use_signal(|| "weight_loss".to_string());
    let mut expectation_status = use_signal(|| "proposed".to_string());
    let mut expectation_priority = use_signal(|| "50".to_string());
    let mut message = use_signal(String::new);

    use_effect(move || {
        apply_document_theme(&theme());
    });

    let reload = move || {
        spawn(async move {
            loading.set(true);
            message.set(String::new());

            let preferences = user::get_user_preferences().await;
            let expectation_result = user::list_health_expectations().await;

            match (preferences, expectation_result) {
                (Ok(preferences), Ok(items)) => {
                    let next_theme = normalize_theme(
                        preferences
                            .theme
                            .unwrap_or_else(|| "system".to_string())
                            .as_str(),
                    );
                    theme.set(next_theme.clone());
                    apply_document_theme(&next_theme);
                    language.set(preferences.language.unwrap_or_else(|| "zh-CN".to_string()));
                    preferred_cuisines.set(preferences.preferred_cuisines);
                    preferred_cuisines_input.set(String::new());
                    avoided_cuisines.set(preferences.avoided_cuisines);
                    avoided_cuisines_input.set(String::new());
                    expectations.set(items);
                }
                (Err(err), _) | (_, Err(err)) => {
                    message.set(format!("加载失败: {err}"));
                }
            }

            loading.set(false);
        });
    };

    use_effect(move || {
        reload();
    });

    let save_preferences = move |_| {
        spawn(async move {
            saving_preferences.set(true);
            message.set(String::new());

            let next_preferred = merge_tags(preferred_cuisines(), &preferred_cuisines_input());
            let next_avoided = merge_tags(avoided_cuisines(), &avoided_cuisines_input());

            let input = user::UpdatePreferencesInput {
                theme: Some(theme()),
                language: Some(language()),
                preferred_cuisines: next_preferred.clone(),
                avoided_cuisines: next_avoided.clone(),
            };

            match user::update_user_preferences(input).await {
                Ok(result) => {
                    let next_theme = normalize_theme(
                        result
                            .theme
                            .unwrap_or_else(|| "system".to_string())
                            .as_str(),
                    );
                    theme.set(next_theme.clone());
                    apply_document_theme(&next_theme);
                    language.set(result.language.unwrap_or_else(|| "zh-CN".to_string()));
                    preferred_cuisines.set(result.preferred_cuisines);
                    preferred_cuisines_input.set(String::new());
                    avoided_cuisines.set(result.avoided_cuisines);
                    avoided_cuisines_input.set(String::new());
                    message.set("偏好已保存".to_string());
                }
                Err(err) => {
                    message.set(format!("保存偏好失败: {err}"));
                }
            }

            saving_preferences.set(false);
        });
    };

    let save_expectation = move |_| {
        spawn(async move {
            saving_expectation.set(true);
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

            match user::upsert_health_expectation(input).await {
                Ok(items) => {
                    expectations.set(items);
                    expectation_id.set(String::new());
                    expectation_title.set(String::new());
                    expectation_summary.set(String::new());
                    expectation_kind.set("weight_loss".to_string());
                    expectation_status.set("proposed".to_string());
                    expectation_priority.set("50".to_string());
                    message.set("健康期望已保存".to_string());
                }
                Err(err) => {
                    message.set(format!("保存健康期望失败: {err}"));
                }
            }

            saving_expectation.set(false);
        });
    };

    let active_count = expectations()
        .iter()
        .filter(|item| item.status == "active")
        .count();
    let proposed_count = expectations()
        .iter()
        .filter(|item| item.status == "proposed")
        .count();
    let preference_count = preferred_cuisines().len() + avoided_cuisines().len();

    rsx! {
        div {
            class: "h-full min-h-0 overflow-y-auto px-4 py-5 pb-28 md:px-8 md:py-8 md:pb-12",
            div {
                class: "mx-auto flex w-full max-w-6xl flex-col gap-6",

                section {
                    class: "relative overflow-hidden rounded-[2rem] border border-border bg-card px-5 py-6 shadow-none md:px-8 md:py-8",
                    div { class: "absolute -right-12 -top-16 h-44 w-44 rounded-full bg-[#FFAD1A]/20 blur-3xl" }
                    div { class: "absolute -bottom-20 left-10 h-52 w-52 rounded-full bg-[#0f7a4d]/10 blur-3xl" }
                    div {
                        class: "relative flex flex-col gap-6 md:flex-row md:items-end md:justify-between",
                        div {
                            class: "max-w-2xl",
                            p { class: "mb-3 text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground", "User graph" }
                            h2 {
                                class: "font-doodle text-4xl font-semibold leading-tight tracking-[-0.9px] text-foreground md:text-5xl",
                                "让偏好和健康期望成为 agent 的稳定上下文"
                            }
                            p {
                                class: "mt-4 text-sm leading-relaxed text-muted-foreground md:text-base",
                                "这里维护用户图谱的显式事实：应用偏好、饮食偏好和健康期望。饮食建议会优先读取这些长期事实。"
                            }
                        }
                        div {
                            class: "grid grid-cols-3 gap-2 rounded-[1.5rem] border border-border bg-background/70 p-2 text-center md:min-w-[320px]",
                            StatPill { label: "偏好", value: preference_count.to_string() }
                            StatPill { label: "Active", value: active_count.to_string() }
                            StatPill { label: "Proposed", value: proposed_count.to_string() }
                        }
                    }
                }

                if !message().is_empty() {
                    div {
                        class: "rounded-2xl border border-border bg-card px-4 py-3 text-sm text-foreground shadow-none",
                        "{message}"
                    }
                }

                div {
                    class: "grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(360px,0.8fr)]",
                    div {
                        class: "flex flex-col gap-6",
                        Card {
                            class: "rounded-[2rem] border border-border bg-card shadow-none",
                            CardHeader {
                                class: "gap-2",
                                CardTitle {
                                    class: "flex items-center gap-2 text-xl font-semibold tracking-[-0.3px]",
                                    Palette { size: 18 }
                                    "偏好工作台"
                                }
                                p { class: "text-sm leading-relaxed text-muted-foreground", "应用偏好保持轻量；饮食偏好以 badge 形式保存，便于 agent 直接读取。" }
                            }
                            CardContent {
                                class: "space-y-5",
                                div {
                                    class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                                    LabeledChoiceGroup {
                                        label: "Theme",
                                        icon: rsx! { Palette { size: 16 } },
                                        value: theme(),
                                        options: vec![
                                            ChoiceOption::new("system", "System"),
                                            ChoiceOption::new("light", "Light"),
                                            ChoiceOption::new("dark", "Dark"),
                                        ],
                                        onchange: move |value: String| {
                                            let value = normalize_theme(&value);
                                            theme.set(value.clone());
                                            apply_document_theme(&value);
                                        },
                                    }
                                    LabeledInput {
                                        label: "Language",
                                        icon: rsx! { Globe { size: 16 } },
                                        value: language(),
                                        placeholder: "zh-CN",
                                        oninput: move |value| language.set(value),
                                    }
                                }
                                div {
                                    class: "grid grid-cols-1 gap-4",
                                    TagListInput {
                                        label: "偏好的菜系 / 口味",
                                        icon: rsx! { Check { size: 16 } },
                                        values: preferred_cuisines(),
                                        draft: preferred_cuisines_input(),
                                        placeholder: "云南菜, 粤菜, 清淡",
                                        oninput: move |value| preferred_cuisines_input.set(value),
                                        oncommit: move |_| {
                                            let merged = merge_tags(preferred_cuisines(), &preferred_cuisines_input());
                                            preferred_cuisines.set(merged);
                                            preferred_cuisines_input.set(String::new());
                                        },
                                        onremove: move |value| {
                                            preferred_cuisines.write().retain(|item| item != &value);
                                        },
                                    }
                                    TagListInput {
                                        label: "忌口 / 避免",
                                        icon: rsx! { Trash2 { size: 16 } },
                                        values: avoided_cuisines(),
                                        draft: avoided_cuisines_input(),
                                        placeholder: "香菜, 蒜头, 腐乳",
                                        oninput: move |value| avoided_cuisines_input.set(value),
                                        oncommit: move |_| {
                                            let merged = merge_tags(avoided_cuisines(), &avoided_cuisines_input());
                                            avoided_cuisines.set(merged);
                                            avoided_cuisines_input.set(String::new());
                                        },
                                        onremove: move |value| {
                                            avoided_cuisines.write().retain(|item| item != &value);
                                        },
                                    }
                                }
                                Button {
                                    class: "w-full rounded-xl bg-foreground text-background shadow-sm hover:opacity-90 md:w-auto",
                                    disabled: saving_preferences() || loading(),
                                    onclick: save_preferences,
                                    if saving_preferences() { "保存中..." } else { "保存偏好" }
                                }
                            }
                        }

                        Card {
                            class: "rounded-[2rem] border border-border bg-card shadow-none",
                            CardHeader {
                                class: "gap-2",
                                CardTitle {
                                    class: "flex items-center gap-2 text-xl font-semibold tracking-[-0.3px]",
                                    Flame { size: 18 }
                                    "健康期望编辑台"
                                }
                                p { class: "text-sm leading-relaxed text-muted-foreground", "健康期望是可生命周期管理的显式意图，不再依赖 profile fallback。" }
                            }
                            CardContent {
                                class: "space-y-4",
                                div {
                                    class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                                    LabeledInput {
                                        label: "ID",
                                        icon: rsx! { Plus { size: 16 } },
                                        value: expectation_id(),
                                        placeholder: "留空则新增",
                                        oninput: move |value| expectation_id.set(value),
                                    }
                                    LabeledInput {
                                        label: "Title",
                                        icon: rsx! { Flame { size: 16 } },
                                        value: expectation_title(),
                                        placeholder: "减脂 / 提神 / 控糖",
                                        oninput: move |value| expectation_title.set(value),
                                    }
                                }
                                LabeledTextarea {
                                    label: "Summary",
                                    icon: rsx! { Check { size: 16 } },
                                    value: expectation_summary(),
                                    placeholder: "描述期望，例如：未来两周晚餐少油少糖，优先高蛋白。",
                                    oninput: move |value| expectation_summary.set(value),
                                }
                                div {
                                    class: "grid grid-cols-1 gap-3 md:grid-cols-3",
                                    LabeledChoiceGroup {
                                        label: "Kind",
                                        icon: rsx! { Flame { size: 16 } },
                                        value: expectation_kind(),
                                        options: vec![
                                            ChoiceOption::new("weight_loss", "减脂"),
                                            ChoiceOption::new("energy_boost", "提神"),
                                            ChoiceOption::new("better_sleep", "睡眠"),
                                            ChoiceOption::new("blood_sugar_control", "控糖"),
                                        ],
                                        onchange: move |value: String| expectation_kind.set(value),
                                    }
                                    LabeledChoiceGroup {
                                        label: "Status",
                                        icon: rsx! { Check { size: 16 } },
                                        value: expectation_status(),
                                        options: vec![
                                            ChoiceOption::new("proposed", "Proposed"),
                                            ChoiceOption::new("active", "Active"),
                                            ChoiceOption::new("archived", "Archived"),
                                        ],
                                        onchange: move |value: String| expectation_status.set(value),
                                    }
                                    LabeledInput {
                                        label: "Priority",
                                        icon: rsx! { Plus { size: 16 } },
                                        value: expectation_priority(),
                                        placeholder: "50",
                                        oninput: move |value| expectation_priority.set(value),
                                    }
                                }
                                Button {
                                    class: "w-full rounded-xl bg-foreground text-background shadow-sm hover:opacity-90 md:w-auto",
                                    disabled: saving_expectation() || loading(),
                                    onclick: save_expectation,
                                    if saving_expectation() { "保存中..." } else { "新增 / 更新健康期望" }
                                }
                            }
                        }
                    }

                    Card {
                        class: "rounded-[2rem] border border-border bg-card shadow-none lg:sticky lg:top-8 lg:self-start",
                        CardHeader {
                            class: "gap-2",
                            div {
                                class: "flex items-center justify-between gap-3",
                                CardTitle { class: "text-xl font-semibold tracking-[-0.3px]", "期望列表" }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "rounded-full border border-border px-3",
                                    onclick: move |_| reload(),
                                    RefreshCw { size: 15 }
                                    "刷新"
                                }
                            }
                            p { class: "text-sm leading-relaxed text-muted-foreground", "点击 Edit 会把条目带回左侧编辑台。" }
                        }
                        CardContent {
                            class: "space-y-3",
                            if loading() {
                                div { class: "rounded-2xl border border-border bg-background/70 px-4 py-6 text-sm text-muted-foreground", "加载中..." }
                            } else if expectations().is_empty() {
                                div { class: "rounded-2xl border border-dashed border-border bg-background/70 px-4 py-8 text-sm leading-relaxed text-muted-foreground", "还没有健康期望。先在左侧创建一个明确的阶段性目标。" }
                            } else {
                                for item in expectations().iter() {
                                    HealthExpectationCard {
                                        item: item.clone(),
                                        on_edit: move |current: user::HealthExpectationDto| {
                                            expectation_id.set(current.id.clone());
                                            expectation_title.set(current.title.clone());
                                            expectation_summary.set(current.summary.clone());
                                            expectation_kind.set(current.kind.clone());
                                            expectation_status.set(current.status.clone());
                                            expectation_priority.set(current.priority.to_string());
                                        },
                                        on_confirm: move |id| {
                                            spawn(async move {
                                                match user::confirm_health_expectation(id).await {
                                                    Ok(items) => {
                                                        expectations.set(items);
                                                        message.set("已确认健康期望".to_string());
                                                    }
                                                    Err(err) => message.set(format!("确认失败: {err}")),
                                                }
                                            });
                                        },
                                        on_delete: move |id| {
                                            spawn(async move {
                                                match user::delete_health_expectation(id).await {
                                                    Ok(items) => {
                                                        expectations.set(items);
                                                        message.set("已删除健康期望".to_string());
                                                    }
                                                    Err(err) => message.set(format!("删除失败: {err}")),
                                                }
                                            });
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ChoiceOption {
    value: &'static str,
    label: &'static str,
}

impl ChoiceOption {
    const fn new(value: &'static str, label: &'static str) -> Self {
        Self { value, label }
    }
}

#[component]
fn StatPill(label: String, value: String) -> Element {
    rsx! {
        div {
            class: "rounded-[1.1rem] border border-border bg-card px-3 py-3",
            div { class: "font-doodle text-2xl font-semibold leading-none tracking-[-0.6px] text-foreground", "{value}" }
            div { class: "mt-1 text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground", "{label}" }
        }
    }
}

#[component]
fn LabeledInput(
    label: String,
    icon: Element,
    value: String,
    placeholder: String,
    oninput: EventHandler<String>,
) -> Element {
    rsx! {
        label {
            class: "flex flex-col gap-2",
            span {
                class: "flex items-center gap-2 text-sm font-medium text-foreground/80",
                {icon}
                "{label}"
            }
            Input {
                class: "rounded-xl border border-border bg-background px-4 py-3 text-sm shadow-none focus:shadow-md",
                value: value,
                placeholder: placeholder,
                oninput: move |e: FormEvent| oninput.call(e.value()),
            }
        }
    }
}

#[component]
fn LabeledTextarea(
    label: String,
    icon: Element,
    value: String,
    placeholder: String,
    oninput: EventHandler<String>,
) -> Element {
    rsx! {
        label {
            class: "flex flex-col gap-2",
            span {
                class: "flex items-center gap-2 text-sm font-medium text-foreground/80",
                {icon}
                "{label}"
            }
            textarea {
                class: "min-h-28 rounded-xl border border-border bg-background px-4 py-3 text-sm leading-relaxed text-foreground outline-none placeholder:text-muted-foreground focus:shadow-md",
                value: value,
                placeholder: placeholder,
                oninput: move |e: FormEvent| oninput.call(e.value()),
            }
        }
    }
}

#[component]
fn LabeledChoiceGroup(
    label: String,
    icon: Element,
    value: String,
    options: Vec<ChoiceOption>,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-2",
            span {
                class: "flex items-center gap-2 text-sm font-medium text-foreground/80",
                {icon}
                "{label}"
            }
            div {
                class: "flex flex-wrap gap-2 rounded-[1.25rem] border border-border bg-background p-2",
                for option in options {
                    Button {
                        key: "{option.value}",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: format!(
                            "rounded-xl px-3 {}",
                            if value == option.value {
                                "bg-foreground text-background hover:opacity-90"
                            } else {
                                "border border-border text-foreground/80 hover:bg-muted"
                            }
                        ),
                        onclick: move |_| onchange.call(option.value.to_string()),
                        "{option.label}"
                    }
                }
            }
        }
    }
}

#[component]
fn TagListInput(
    label: String,
    icon: Element,
    values: Vec<String>,
    draft: String,
    placeholder: String,
    oninput: EventHandler<String>,
    oncommit: EventHandler<()>,
    onremove: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-2",
            span {
                class: "flex items-center gap-2 text-sm font-medium text-foreground/80",
                {icon}
                "{label}"
            }
            div {
                class: "rounded-[1.5rem] border border-border bg-background p-3",
                div {
                    class: "mb-3 flex min-h-8 flex-wrap gap-2",
                    if values.is_empty() {
                        span {
                            class: "rounded-full border border-dashed border-border px-3 py-1 text-xs text-muted-foreground",
                            "暂无条目"
                        }
                    } else {
                        for item in values {
                            button {
                                key: "{item}",
                                r#type: "button",
                                class: "inline-flex items-center gap-2 rounded-full bg-foreground px-3 py-1 text-xs font-medium text-background shadow-sm",
                                onclick: {
                                    let item = item.clone();
                                    move |_| onremove.call(item.clone())
                                },
                                span { "{item}" }
                                span { class: "text-background/70", "x" }
                            }
                        }
                    }
                }
                div {
                    class: "flex flex-col gap-2 sm:flex-row sm:items-center",
                    Input {
                        class: "flex-1 rounded-xl border border-border bg-card px-4 py-3 text-sm shadow-none",
                        value: draft,
                        placeholder: placeholder,
                        oninput: move |e: FormEvent| oninput.call(e.value()),
                        onblur: move |_| oncommit.call(()),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                oncommit.call(());
                            }
                        },
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "rounded-xl border border-border px-4",
                        onclick: move |_| oncommit.call(()),
                        "添加"
                    }
                }
                p {
                    class: "mt-3 text-xs leading-relaxed text-muted-foreground",
                    "支持使用英文逗号、中文逗号、分号或换行一次输入多个条目。"
                }
            }
        }
    }
}

#[component]
fn HealthExpectationCard(
    item: user::HealthExpectationDto,
    on_edit: EventHandler<user::HealthExpectationDto>,
    on_confirm: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    let status_badge = match item.status.as_str() {
        "active" => "bg-[#0f7a4d] text-white",
        "archived" => "bg-muted text-muted-foreground",
        _ => "bg-[#b86b10] text-white",
    };
    let is_active = item.status == "active";
    let expectation_id = item.id.clone();
    let expectation_id_for_delete = expectation_id.clone();
    let item_for_edit = item.clone();

    rsx! {
        div {
            class: "rounded-[1.5rem] border border-border bg-background p-4",
            div {
                class: "flex items-start justify-between gap-3",
                div {
                    class: "min-w-0 space-y-1",
                    h3 { class: "text-base font-semibold tracking-[-0.2px] text-foreground", "{item.title}" }
                    p { class: "text-sm leading-relaxed text-muted-foreground", "{item.summary}" }
                }
                span {
                    class: format!("shrink-0 rounded-full px-3 py-1 text-xs font-semibold {}", status_badge),
                    "{item.status}"
                }
            }
            div {
                class: "mt-4 flex flex-wrap gap-2 text-xs text-muted-foreground",
                span { class: "rounded-full border border-border bg-card px-2 py-1", "kind: {item.kind}" }
                span { class: "rounded-full border border-border bg-card px-2 py-1", "priority: {item.priority}" }
                span { class: "rounded-full border border-border bg-card px-2 py-1", "{item.source}" }
            }
            div {
                class: "mt-4 grid grid-cols-3 gap-2",
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "rounded-xl border border-border",
                    onclick: move |_| on_edit.call(item_for_edit.clone()),
                    "Edit"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "rounded-xl border border-border",
                    onclick: move |_| on_confirm.call(expectation_id.clone()),
                    disabled: is_active,
                    "Confirm"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "rounded-xl border border-destructive/30 text-destructive",
                    onclick: move |_| on_delete.call(expectation_id_for_delete.clone()),
                    "Delete"
                }
            }
        }
    }
}

fn parse_csv(input: &str) -> Vec<String> {
    input
        .replace(['，', ';', '；'], ",")
        .lines()
        .flat_map(|line| line.split(','))
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn merge_tags(existing: Vec<String>, draft: &str) -> Vec<String> {
    let mut merged = existing;

    for item in parse_csv(draft) {
        if !merged.iter().any(|existing| existing == &item) {
            merged.push(item);
        }
    }

    merged
}

fn normalize_theme(value: &str) -> String {
    match value.trim() {
        "light" => "light",
        "dark" => "dark",
        _ => "system",
    }
    .to_string()
}

fn apply_document_theme(theme: &str) {
    let script = match theme {
        "light" => r#"document.documentElement.setAttribute("data-theme", "light");"#,
        "dark" => r#"document.documentElement.setAttribute("data-theme", "dark");"#,
        _ => r#"document.documentElement.removeAttribute("data-theme");"#,
    };
    document::eval(script);
}
