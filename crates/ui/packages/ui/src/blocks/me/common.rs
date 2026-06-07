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
        "rounded-2xl border border-border bg-card px-4 py-3 transition-colors hover:bg-muted"
    } else {
        "rounded-2xl border border-border bg-card px-4 py-3"
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
            div { class: "text-2xl font-medium tracking-tight text-foreground", "{value}" }
            div { class: "mt-1 text-[11px] font-medium uppercase tracking-widest text-muted-foreground", "{label}" }
        }
    }
}

#[component]
pub fn BlockMessage(message: String) -> Element {
    rsx! {
        if !message.is_empty() {
            div { class: "rounded-xl border border-border bg-card px-4 py-3 text-sm text-foreground", "{message}" }
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
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground", {icon} "{label}" }
            Input {
                class: "rounded-md border border-border bg-card px-3 py-2.5 text-sm shadow-none transition-all hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
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
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground", {icon} "{label}" }
            textarea {
                class: "min-h-28 rounded-md border border-border bg-card px-3 py-2.5 text-sm leading-relaxed text-foreground outline-none transition-all placeholder:text-muted-foreground hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
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
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground", {icon} "{label}" }
            div { class: "flex flex-wrap gap-2 rounded-xl border border-border bg-card p-2",
                for option in options {
                    Button {
                        key: "{label}:{option.value}",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: format!(
                            "rounded-md px-3 transition-colors {}",
                            if value() == option.value {
                                "bg-foreground text-background shadow-xs hover:opacity-90"
                            } else {
                                "border border-transparent text-muted-foreground hover:bg-muted hover:text-foreground"
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
            span { class: "flex items-center gap-2 text-sm font-medium text-foreground", {icon} "{label}" }
            div { class: "rounded-xl border border-border bg-card p-3",
                div { class: "mb-3 flex min-h-8 flex-wrap gap-2",
                    if values.is_empty() {
                        span { class: "rounded-md border border-dashed border-border px-3 py-1 text-xs text-muted-foreground", "暂无条目" }
                    } else {
                        for item in values {
                            button {
                                key: "{item}",
                                r#type: "button",
                                class: "inline-flex items-center gap-2 rounded-md bg-foreground px-3 py-1 text-xs font-medium text-background shadow-xs transition-opacity hover:opacity-90",
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
                        class: "min-w-0 flex-1 rounded-md border border-border bg-card px-3 py-2 text-sm shadow-none transition-all hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
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
                        class: "shrink-0 rounded-md border border-border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                        onclick: move |_| oncommit.call(()),
                        Plus { size: 16 }
                    }
                }
            }
            p { class: "px-1 text-xs leading-relaxed text-muted-foreground", "支持使用中英文逗号、分号或回车确认输入多个条目。" }
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
