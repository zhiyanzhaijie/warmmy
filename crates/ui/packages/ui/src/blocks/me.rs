use dioxus::prelude::*;
use dioxus_icons::lucide::{Bell, ChevronRight, CircleQuestionMark, LogOut, Shield, User};

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::Card;

#[derive(Clone, Copy, PartialEq)]
enum SettingsIcon {
    User,
    Bell,
    Shield,
    CircleQuestionMark,
}

#[derive(Clone, Copy, PartialEq)]
struct SettingsItem {
    icon: SettingsIcon,
    label: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
struct SettingsGroup {
    title: &'static str,
    items: &'static [SettingsItem],
}

const ACCOUNT_ITEMS: &[SettingsItem] = &[
    SettingsItem {
        icon: SettingsIcon::User,
        label: "Personal Data",
    },
    SettingsItem {
        icon: SettingsIcon::Bell,
        label: "Notifications",
    },
];

const SETTINGS_ITEMS: &[SettingsItem] = &[
    SettingsItem {
        icon: SettingsIcon::Shield,
        label: "Privacy and Security",
    },
    SettingsItem {
        icon: SettingsIcon::CircleQuestionMark,
        label: "Help and Support",
    },
];

const SETTINGS_GROUPS: &[SettingsGroup] = &[
    SettingsGroup {
        title: "Account",
        items: ACCOUNT_ITEMS,
    },
    SettingsGroup {
        title: "Settings",
        items: SETTINGS_ITEMS,
    },
];

fn settings_icon(icon: SettingsIcon) -> Element {
    match icon {
        SettingsIcon::User => rsx! { User { size: 20 } },
        SettingsIcon::Bell => rsx! { Bell { size: 20 } },
        SettingsIcon::Shield => rsx! { Shield { size: 20 } },
        SettingsIcon::CircleQuestionMark => rsx! { CircleQuestionMark { size: 20 } },
    }
}

#[component]
pub fn MeBlock() -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full max-w-2xl mx-auto w-full px-6 py-6 md:py-12",
            h2 {
                class: "font-doodle text-3xl font-bold text-foreground mb-8",
                "Settings"
            }

            div {
                class: "space-y-8",
                for group in SETTINGS_GROUPS.iter() {
                    div {
                        key: "{group.title}",
                        h3 {
                            class: "text-xs font-bold text-muted-foreground uppercase tracking-wider mb-3 px-2",
                            "{group.title}"
                        }
                        Card {
                            class: "bg-card rounded-[1.5rem] border border-border overflow-hidden shadow-sm",
                            for (idx, item) in group.items.iter().enumerate() {
                                div {
                                    key: "{item.label}",
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        class: "w-full flex items-center justify-between p-4 bg-card hover:bg-muted/30 active:bg-muted transition-colors rounded-none",
                                        div {
                                            class: "flex items-center gap-4",
                                            div {
                                                class: "bg-muted p-2 rounded-xl text-foreground",
                                                {settings_icon(item.icon)}
                                            }
                                            span {
                                                class: "font-semibold text-foreground",
                                                "{item.label}"
                                            }
                                        }
                                        ChevronRight { size: 20, class: "text-muted-foreground" }
                                    }
                                    if idx != group.items.len() - 1 {
                                        div { class: "h-[1px] bg-border ml-16 mr-4" }
                                    }
                                }
                            }
                        }
                    }
                }

                Button {
                    variant: ButtonVariant::Ghost,
                    class: "mt-8 w-full flex items-center justify-center gap-2 p-4 bg-card border border-destructive/30 rounded-[1.5rem] text-destructive font-bold hover:bg-destructive/10 active:bg-destructive/20 transition-colors shadow-sm",
                    LogOut { size: 20 }
                    "Log Out"
                }
            }
        }
    }
}
