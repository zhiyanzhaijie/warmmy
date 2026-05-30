use chrono::{Duration as ChronoDuration, NaiveDate};
use dioxus::prelude::*;
use dioxus_icons::lucide::CalendarDays;
use std::collections::HashSet;

use crate::today_session_id;

#[component]
pub(super) fn SessionStrip(user_id: String, active_session_id: String) -> Element {
    let sessions = use_resource(move || {
        let request_user_id = user_id.clone();
        async move {
            api::conversation::list_user_sessions(request_user_id)
                .await
                .unwrap_or_default()
        }
    });

    let session_list = sessions.read().clone().unwrap_or_default();
    let session_days: HashSet<String> = session_list.into_iter().collect();
    let days = recent_session_days(7)
        .into_iter()
        .map(|day| {
            let has_session = session_days.contains(&day);
            (day, has_session)
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            class: "overflow-x-auto px-4 pb-2 pt-1 md:px-5 hide-scrollbar",
            div {
                class: "flex min-w-max gap-1.5 rounded-[1.15rem] bg-background/75 p-1",
                for (day, has_session) in days {
                    SessionChip {
                        session_id: day,
                        active_session_id: active_session_id.clone(),
                        has_session,
                    }
                }
            }
        }
    }
}

#[component]
fn SessionChip(session_id: String, active_session_id: String, has_session: bool) -> Element {
    let is_active = session_id == active_session_id;
    let label = session_label(&session_id);

    if is_active {
        rsx! {
            button {
                r#type: "button",
                disabled: true,
                class: "inline-flex items-center gap-1.5 rounded-full bg-foreground px-2.5 py-1.5 text-xs font-semibold text-background shadow-sm",
                CalendarDays { size: 13 }
                "{label}"
            }
        }
    } else if has_session {
        let sid = session_id.clone();
        rsx! {
            button {
                r#type: "button",
                onclick: move |_| { navigator().push(format!("/{sid}")); },
                class: "inline-flex items-center gap-1.5 rounded-full border border-transparent px-2.5 py-1.5 text-xs font-semibold text-muted-foreground transition-colors hover:border-border hover:bg-card hover:text-foreground",
                CalendarDays { size: 13 }
                "{label}"
            }
        }
    } else {
        rsx! {
            button {
                r#type: "button",
                disabled: true,
                class: "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1.5 text-xs font-semibold text-muted-foreground opacity-35",
                CalendarDays { size: 13 }
                "{label}"
            }
        }
    }
}

fn recent_session_days(count: i64) -> Vec<String> {
    let today = today_session_id();
    let Some(today) = NaiveDate::parse_from_str(&today, "%Y-%m-%d").ok() else {
        return vec![today_session_id()];
    };

    (0..count)
        .map(|offset| {
            (today - ChronoDuration::days(offset))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect()
}

fn session_label(session_id: &str) -> String {
    if session_id == today_session_id() {
        "今天".to_string()
    } else {
        session_id
            .rsplit_once('-')
            .map(|(prefix, day)| {
                prefix
                    .rsplit_once('-')
                    .map(|(_, month)| format!("{month}/{day}"))
                    .unwrap_or_else(|| session_id.to_string())
            })
            .unwrap_or_else(|| session_id.to_string())
    }
}
