use dioxus::prelude::*;
use dioxus_icons::lucide::{Plus, X};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::input::Input;

#[derive(Clone, PartialEq)]
pub struct ChoiceOption {
    pub value: &'static str,
    pub label: &'static str,
}

impl ChoiceOption {
    pub const fn new(value: &'static str, label: &'static str) -> Self {
        Self { value, label }
    }
}

#[component]
pub fn StatPill(
    label: String,
    value: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let class = if onclick.is_some() {
        "rounded-[1.1rem] border border-border bg-card px-3 py-3 transition hover:bg-muted/50"
    } else {
        "rounded-[1.1rem] border border-border bg-card px-3 py-3"
    };

    rsx! {
        button {
            r#type: "button",
            class,
            disabled: onclick.is_none(),
            onclick: move |event| {
                if let Some(action) = onclick {
                    action.call(event);
                }
            },
            div { class: "font-doodle text-2xl font-semibold leading-none tracking-[-0.6px] text-foreground", "{value}" }
            div { class: "mt-1 text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground", "{label}" }
        }
    }
}

#[component]
pub fn BlockMessage(message: String) -> Element {
    rsx! {
        if !message.is_empty() {
            div { class: "rounded-2xl border border-border bg-background/70 px-4 py-3 text-sm text-foreground", "{message}" }
        }
    }
}

#[component]
pub fn MiniTag(label: String) -> Element {
    rsx! {
        span { class: "inline-flex max-w-full items-center truncate rounded-full border border-border bg-card px-3 py-1 text-xs font-medium text-muted-foreground",
            "{label}"
        }
    }
}

#[component]
pub fn LabeledInput(
    label: String,
    icon: Element,
    mut value: Signal<String>,
    placeholder: String,
) -> Element {
    rsx! {
        label { class: "flex flex-col gap-2",
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground/80", {icon} "{label}" }
            Input {
                class: "rounded-xl border border-border bg-background px-4 py-3 text-sm shadow-none focus:shadow-md",
                value: value(),
                placeholder,
                oninput: move |e: FormEvent| value.set(e.value()),
            }
        }
    }
}

#[component]
pub fn LabeledTextarea(
    label: String,
    icon: Element,
    mut value: Signal<String>,
    placeholder: String,
) -> Element {
    rsx! {
        label { class: "flex flex-col gap-2",
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground/80", {icon} "{label}" }
            textarea {
                class: "min-h-28 rounded-xl border border-border bg-background px-4 py-3 text-sm leading-relaxed text-foreground outline-none placeholder:text-muted-foreground focus:shadow-md",
                value: value(),
                placeholder,
                oninput: move |e: FormEvent| value.set(e.value()),
            }
        }
    }
}

#[component]
pub fn LabeledChoiceGroup(
    label: String,
    icon: Element,
    mut value: Signal<String>,
    options: Vec<ChoiceOption>,
    #[props(default)] onselect: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-2",
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground/80", {icon} "{label}" }
            div { class: "flex flex-wrap gap-2 rounded-[1.25rem] border border-border bg-background p-2",
                for option in options {
                    Button {
                        key: "{label}:{option.value}",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: format!(
                            "rounded-xl px-3 {}",
                            if value() == option.value {
                                "bg-foreground text-background hover:opacity-90"
                            } else {
                                "border border-border text-foreground/80 hover:bg-muted"
                            }
                        ),
                        onclick: move |_| {
                            let next = option.value.to_string();
                            value.set(next.clone());
                            if let Some(onselect) = onselect {
                                onselect.call(next);
                            }
                        },
                        "{option.label}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn TagListInput(
    label: String,
    icon: Element,
    values: Vec<String>,
    mut draft: Signal<String>,
    placeholder: String,
    oncommit: EventHandler<()>,
    onremove: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-2",
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground/80", {icon} "{label}" }
            div { class: "rounded-[1.25rem] border border-border bg-background p-3",
                div { class: "mb-3 flex min-h-8 flex-wrap gap-2",
                    if values.is_empty() {
                        span { class: "rounded-full border border-dashed border-border px-3 py-1 text-xs text-muted-foreground", "暂无条目" }
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
                                X { size: 12 }
                            }
                        }
                    }
                }
                div { class: "flex flex-col gap-2 sm:flex-row sm:items-center",
                    Input {
                        class: "min-w-0 flex-1 rounded-xl border border-border bg-card px-4 py-2.5 text-sm shadow-none",
                        value: draft(),
                        placeholder,
                        oninput: move |e: FormEvent| draft.set(e.value()),
                        onblur: move |_| oncommit.call(()),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                oncommit.call(());
                            }
                        },
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        class: "shrink-0 rounded-xl border border-border",
                        onclick: move |_| oncommit.call(()),
                        Plus { size: 16 }
                    }
                }
            }
            p { class: "px-1 text-xs leading-relaxed text-muted-foreground/75", "支持使用英文逗号、中文逗号、分号或换行一次输入多个条目。" }
        }
    }
}

pub fn parse_csv(input: &str) -> Vec<String> {
    input
        .replace(['，', ';', '；'], ",")
        .lines()
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn merge_tags(existing: Vec<String>, draft: &str) -> Vec<String> {
    let mut merged = existing;

    for item in parse_csv(draft) {
        if !merged.iter().any(|existing| existing == &item) {
            merged.push(item);
        }
    }

    merged
}

pub fn normalize_theme(value: &str) -> String {
    match value.trim() {
        "light" => "light",
        "dark" => "dark",
        _ => "system",
    }
    .to_string()
}

pub fn apply_document_theme(theme: &str) {
    let script = match theme {
        "light" => r#"document.documentElement.setAttribute("data-theme", "light");"#,
        "dark" => r#"document.documentElement.setAttribute("data-theme", "dark");"#,
        _ => r#"document.documentElement.removeAttribute("data-theme");"#,
    };
    document::eval(script);
}
