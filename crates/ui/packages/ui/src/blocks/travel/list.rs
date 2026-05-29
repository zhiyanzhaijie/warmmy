use api::meal;
use dioxus::prelude::*;
use dioxus_icons::lucide::{CalendarDays, ChevronRight, Map, TrendingUp};

use crate::blocks::CurrentUserContext;

use super::metrics::{compact_number, date_label, parse_metrics};

#[component]
pub fn TravelListBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let user_id = (current_user.user_id)();
    let mut items = use_signal(Vec::<meal::MealDaySummaryDTO>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);

    use_effect(move || {
        let request_user_id = user_id.clone();
        spawn(async move {
            loading.set(true);
            error.set(String::new());
            match meal::list_meal_day_summaries(request_user_id).await {
                Ok(next) => items.set(next),
                Err(err) => error.set(format!("加载旅程失败: {err}")),
            }
            loading.set(false);
        });
    });

    let summaries = items();
    let count = summaries.len();
    let average_score = if summaries.is_empty() {
        0.0
    } else {
        summaries.iter().map(|item| item.overall_score).sum::<f32>() / count as f32
    };
    let best_score = summaries
        .iter()
        .map(|item| item.overall_score)
        .fold(0.0_f32, f32::max);

    rsx! {
        div { class: "h-full min-h-0 overflow-y-auto px-4 py-5 pb-28 md:px-8 md:py-8 md:pb-12",
            div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6",
                section { class: "rounded-xl border border-border bg-card p-5 md:p-8",
                    div { class: "flex flex-col gap-6 md:flex-row md:items-end md:justify-between",
                        div { class: "max-w-2xl",
                            p { class: "mb-3 flex items-center gap-2 text-sm text-muted-foreground",
                                Map { size: 16 }
                                "Daily travel"
                            }
                            h1 { class: "font-sans text-5xl font-semibold leading-none tracking-[-1.2px] text-foreground md:text-6xl",
                                "饮食旅程"
                            }
                            p { class: "mt-4 text-base leading-relaxed text-muted-foreground md:text-lg",
                                "每天敲定后的 summary 会沉淀在这里，按时间线回看你的饮食节奏、分数和变化。"
                            }
                        }
                        div { class: "grid grid-cols-3 gap-2 md:min-w-[22rem]",
                            StatTile { label: "天数".to_string(), value: count.to_string(), icon: "calendar".to_string() }
                            StatTile { label: "平均".to_string(), value: compact_number(average_score), icon: "trend".to_string() }
                            StatTile { label: "最佳".to_string(), value: compact_number(best_score), icon: "score".to_string() }
                        }
                    }
                }

                if !error().is_empty() {
                    div { class: "rounded-xl border border-border bg-card px-4 py-3 text-sm text-destructive",
                        "{error}"
                    }
                } else if loading() {
                    TimelineSkeleton {}
                } else if summaries.is_empty() {
                    EmptyTimeline {}
                } else {
                    div { class: "relative flex flex-col gap-3 pb-4",
                        div { class: "absolute bottom-5 left-[1.45rem] top-5 w-px bg-border md:left-[1.7rem]" }
                        for summary in summaries {
                            TimelineItem { summary }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatTile(label: String, value: String, icon: String) -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-background px-3 py-3 md:px-4",
            div { class: "mb-2 flex h-8 w-8 items-center justify-center rounded-full border border-border bg-card text-muted-foreground",
                if icon == "calendar" {
                    CalendarDays { size: 16 }
                } else if icon == "trend" {
                    TrendingUp { size: 16 }
                } else {
                    Map { size: 16 }
                }
            }
            p { class: "text-xs text-muted-foreground", "{label}" }
            p { class: "mt-1 text-2xl font-semibold tracking-[-0.4px] text-foreground", "{value}" }
        }
    }
}

#[component]
fn TimelineItem(summary: meal::MealDaySummaryDTO) -> Element {
    let metrics = parse_metrics(&summary);
    let score = compact_number(summary.overall_score);
    let nutrition_score = compact_number(summary.nutrition_score);
    let expectation_score = compact_number(summary.expectation_match_score);
    let calories = compact_number(metrics.total_nutrition.calories);

    rsx! {
        Link {
            to: format!("/travel/{}", summary.session_id),
            class: "group relative grid grid-cols-[3rem_1fr] gap-3 md:grid-cols-[3.5rem_1fr]",
            div { class: "relative z-10 flex h-12 w-12 items-center justify-center rounded-full border border-border bg-background text-xs text-muted-foreground md:h-14 md:w-14",
                "{date_label(&summary.session_id)}"
            }
            article { class: "rounded-xl border border-border bg-card p-4 transition-colors group-hover:bg-accent md:p-5",
                div { class: "flex items-start justify-between gap-4",
                    div { class: "min-w-0 flex-1",
                        div { class: "flex flex-wrap items-center gap-2",
                            p { class: "text-sm text-foreground", "{summary.session_id}" }
                            span { class: "rounded-full border border-border bg-background px-2.5 py-1 text-xs text-muted-foreground",
                                "{metrics.meal_count} meals"
                            }
                        }
                        p { class: "mt-2 line-clamp-2 text-sm leading-relaxed text-muted-foreground",
                            "{summary.content}"
                        }
                    }
                    div { class: "flex shrink-0 items-center gap-2",
                        div { class: "text-right",
                            p { class: "text-xs text-muted-foreground", "总分" }
                            p { class: "text-3xl font-semibold leading-none tracking-[-0.6px] text-foreground",
                                "{score}"
                            }
                        }
                        ChevronRight { size: 18, class: "text-muted-foreground" }
                    }
                }
                div { class: "mt-4 grid grid-cols-3 gap-2 text-xs",
                    MetricPill { label: "营养".to_string(), value: nutrition_score }
                    MetricPill { label: "期望".to_string(), value: expectation_score }
                    MetricPill { label: "热量".to_string(), value: calories }
                }
            }
        }
    }
}

#[component]
fn MetricPill(label: String, value: String) -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-background px-3 py-2",
            p { class: "text-xs text-muted-foreground", "{label}" }
            p { class: "mt-0.5 text-sm text-foreground", "{value}" }
        }
    }
}

#[component]
fn TimelineSkeleton() -> Element {
    rsx! {
        div { class: "flex flex-col gap-3",
            for _ in 0..4 {
                div { class: "h-28 animate-pulse rounded-xl border border-border bg-muted" }
            }
        }
    }
}

#[component]
fn EmptyTimeline() -> Element {
    rsx! {
        div { class: "rounded-xl border border-dashed border-border bg-card px-5 py-10 text-center",
            CalendarDays { size: 32, class: "mx-auto text-muted-foreground" }
            h2 { class: "mt-4 text-3xl font-semibold tracking-[-0.6px] text-foreground", "还没有旅程记录" }
            p { class: "mx-auto mt-2 max-w-md text-sm leading-relaxed text-muted-foreground",
                "在 chat 中完成某一天的餐食敲定和 summary 后，它会出现在这里。"
            }
        }
    }
}
