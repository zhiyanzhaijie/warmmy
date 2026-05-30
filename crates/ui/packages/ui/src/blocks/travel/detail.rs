use api::meal;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowLeft, CalendarDays, Flame, ListChecks};

use crate::blocks::CurrentUserContext;
use crate::components::common::MarkdownContent;

use super::metrics::{compact_number, detail_date_label, parse_metrics};

#[component]
pub fn TravelDetailBlock(summary_id: String) -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let user_id = (current_user.user_id)();
    let mut summary = use_signal(|| Option::<meal::MealDaySummaryDTO>::None);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);
    let nav = navigator();
    let detail_user_id = user_id.clone();
    let detail_summary_id = summary_id.clone();

    use_effect(move || {
        let request_user_id = detail_user_id.clone();
        let request_summary_id = detail_summary_id.clone();
        spawn(async move {
            loading.set(true);
            error.set(String::new());
            match meal::get_meal_day_summary(request_user_id, request_summary_id).await {
                Ok(next) => summary.set(next),
                Err(err) => error.set(format!("加载 summary 失败: {err}")),
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "relative flex h-full min-h-0 flex-col px-4 py-4 md:px-8 md:py-8",
            div { class: "relative mx-auto flex h-full min-h-0 w-full max-w-5xl flex-col gap-4 md:gap-5",
                div { class: "flex items-center justify-between gap-4",
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            nav.push("/travel");
                        },
                        class: "inline-flex items-center gap-2 rounded-full border border-border bg-card/80 px-3 py-2 text-sm text-foreground transition-opacity active:opacity-80",
                        ArrowLeft { size: 16 }
                        "返回"
                    }
                    p { class: "text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Travel summary" }
                }

                div { class: "min-h-0 flex-1 overflow-y-auto pb-28 md:pb-12",
                    if !error().is_empty() {
                        div { class: "rounded-xl border border-border bg-card px-4 py-3 text-sm text-destructive",
                            "{error}"
                        }
                    } else if loading() {
                        DetailSkeleton {}
                    } else if let Some(item) = summary() {
                        SummaryDetail {
                            summary: item.clone(),
                            user_id: user_id.clone(),
                            session_id: item.session_id.clone(),
                        }
                    } else {
                        div { class: "rounded-xl border border-dashed border-border bg-card px-5 py-10 text-center",
                            CalendarDays { size: 32, class: "mx-auto text-muted-foreground" }
                            h1 { class: "mt-4 text-3xl font-semibold tracking-[-0.6px] text-foreground", "没有找到这一天" }
                            p { class: "mx-auto mt-2 max-w-md text-sm leading-relaxed text-muted-foreground",
                                "这个 summary 可能还没有生成，或者当前用户下没有对应记录。"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SummaryDetail(summary: meal::MealDaySummaryDTO, user_id: String, session_id: String) -> Element {
    let metrics = parse_metrics(&summary);
    let total = compact_number(summary.overall_score);
    let nutrition = compact_number(summary.nutrition_score);
    let expectation = compact_number(summary.expectation_match_score);
    let calories = compact_number(metrics.total_nutrition.calories);
    let protein = compact_number(metrics.total_nutrition.protein_g);
    let fat = compact_number(metrics.total_nutrition.fat_g);
    let carbs = compact_number(metrics.total_nutrition.carbs_g);
    let summary_content = use_signal(|| summary.content.clone());

    rsx! {
        section { class: "rounded-2xl border border-border bg-card/85 p-4 md:p-5",
            div { class: "flex items-start justify-between gap-4",
                p { class: "inline-flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1 text-xs text-muted-foreground",
                    CalendarDays { size: 14 }
                    "{detail_date_label(&summary.session_id)}"
                }
                p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Scores" }
            }
            div { class: "mt-5 grid grid-cols-[1.15fr_1fr] gap-2 md:grid-cols-[1.2fr_repeat(3,0.8fr)]",
                div { class: "rounded-xl border border-border bg-background/70 px-4 py-4",
                    p { class: "text-[11px] text-muted-foreground", "总分" }
                    p { class: "mt-1 text-5xl font-semibold leading-none tracking-[-1px] text-foreground md:text-6xl",
                        "{total}"
                    }
                }
                MiniSignal { label: "营养".to_string(), value: nutrition }
                MiniSignal { label: "期望".to_string(), value: expectation }
                MiniSignal { label: "餐次".to_string(), value: metrics.meal_count.to_string() }
            }
            div { class: "mt-3 grid grid-cols-2 gap-2 md:grid-cols-4",
                NutritionTile { label: "热量".to_string(), value: calories, unit: "kcal".to_string() }
                NutritionTile { label: "蛋白质".to_string(), value: protein, unit: "g".to_string() }
                NutritionTile { label: "脂肪".to_string(), value: fat, unit: "g".to_string() }
                NutritionTile { label: "碳水".to_string(), value: carbs, unit: "g".to_string() }
            }
        }

        MealLogsSection {
            user_id,
            session_id,
        }

        section { class: "rounded-2xl border border-border bg-card/85 p-4 md:p-5",
            div { class: "mb-4 flex items-center justify-between gap-4",
                p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Warmmy says" }
                Flame { size: 20, class: "text-muted-foreground" }
            }
            MarkdownContent {
                src: summary_content,
                class: "text-sm leading-relaxed text-foreground md:text-[0.95rem]".to_string()
            }
        }
    }
}

#[component]
fn MiniSignal(label: String, value: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border bg-background/70 px-3 py-3",
            p { class: "text-[11px] text-muted-foreground", "{label}" }
            p { class: "mt-1 text-2xl font-semibold leading-none tracking-[-0.4px] text-foreground", "{value}" }
        }
    }
}

#[component]
fn MealLogsSection(user_id: String, session_id: String) -> Element {
    let mut logs = use_signal(Vec::<meal::MealRecordDTO>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);

    use_effect(move || {
        let request_user_id = user_id.clone();
        let request_session_id = session_id.clone();
        spawn(async move {
            loading.set(true);
            error.set(String::new());
            match meal::list_meal_logs(request_user_id, request_session_id).await {
                Ok(next) => logs.set(next),
                Err(err) => error.set(format!("加载 meal logs 失败: {err}")),
            }
            loading.set(false);
        });
    });

    rsx! {
        section { class: "rounded-2xl border border-border bg-card/75 p-4 md:p-5",
            div { class: "flex items-start justify-between gap-4",
                div {
                    p { class: "mb-2 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground",
                        ListChecks { size: 16 }
                        "Meal logs"
                    }
                    h2 { class: "text-3xl font-semibold tracking-[-0.6px] text-foreground", "当天餐食" }
                }
                div { class: "rounded-full border border-border bg-background/70 px-3 py-1 text-sm font-semibold text-foreground",
                    "{logs().len()}"
                }
            }

            if !error().is_empty() {
                div { class: "mt-5 rounded-lg border border-border bg-background px-4 py-3 text-sm text-destructive",
                    "{error}"
                }
            } else if loading() {
                div { class: "mt-4 grid grid-cols-1 gap-2",
                    for _ in 0..3 {
                        div { class: "h-28 animate-pulse rounded-lg border border-border bg-muted" }
                    }
                }
            } else if logs().is_empty() {
                div { class: "mt-5 rounded-lg border border-dashed border-border bg-background px-4 py-8 text-center text-sm text-muted-foreground",
                    "这一天还没有保存的 meal log。"
                }
            } else {
                div { class: "mt-5 grid grid-cols-1 gap-3",
                    for item in logs() {
                        MealLogCard { item }
                    }
                }
            }
        }
    }
}

#[component]
fn MealLogCard(item: meal::MealRecordDTO) -> Element {
    let calories = compact_number(item.nutrition.calories);
    let protein = compact_number(item.nutrition.protein_g);
    let fat = compact_number(item.nutrition.fat_g);
    let carbs = compact_number(item.nutrition.carbs_g);

    rsx! {
        article { class: "overflow-hidden rounded-xl border border-border bg-background/70",
            div { class: "grid grid-cols-1 md:grid-cols-[0.8fr_1.2fr]",
                div { class: "min-w-0",
                    div { class: "flex h-full flex-col justify-between gap-4 px-4 py-4",
                        div { class: "flex items-center gap-2",
                            span { class: "rounded-full border border-border bg-card px-3 py-1 text-sm font-semibold text-foreground",
                                "{day_cycle_label(&item.day_cycle)}"
                            }
                            span { class: "text-sm text-muted-foreground", "{item.foods.len()} foods" }
                        }
                        div { class: "mt-3 flex flex-wrap gap-2",
                            for food in item.foods.clone() {
                                span { class: "rounded-lg border border-border bg-card/80 px-3 py-1.5 text-sm text-foreground",
                                    "{food.name}"
                                    span { class: "ml-1 text-muted-foreground",
                                        "{compact_number(food.quantity)}{food.unit}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "grid grid-cols-4 border-t border-border bg-card/60 md:border-l md:border-t-0",
                    TinyNutrition { label: "热量".to_string(), value: calories, unit: "kcal".to_string() }
                    TinyNutrition { label: "蛋白".to_string(), value: protein, unit: "g".to_string() }
                    TinyNutrition { label: "脂肪".to_string(), value: fat, unit: "g".to_string() }
                    TinyNutrition { label: "碳水".to_string(), value: carbs, unit: "g".to_string() }
                }
            }
        }
    }
}

#[component]
fn TinyNutrition(label: String, value: String, unit: String) -> Element {
    rsx! {
        div { class: "border-r border-border px-2 py-3 text-center last:border-r-0 md:py-4",
            p { class: "text-[11px] text-muted-foreground", "{label}" }
            p { class: "mt-1 text-sm text-foreground",
                "{value}"
                span { class: "ml-0.5 text-[11px] text-muted-foreground", "{unit}" }
            }
        }
    }
}

#[component]
fn NutritionTile(label: String, value: String, unit: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border bg-background/70 p-4",
            p { class: "text-sm text-muted-foreground", "{label}" }
            p { class: "mt-2 text-2xl font-semibold tracking-[-0.4px] text-foreground",
                "{value}"
                span { class: "ml-1 text-sm font-normal text-muted-foreground", "{unit}" }
            }
        }
    }
}

#[component]
fn DetailSkeleton() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            div { class: "h-48 animate-pulse rounded-xl border border-border bg-muted" }
            div { class: "h-56 animate-pulse rounded-xl border border-border bg-muted" }
            div { class: "h-36 animate-pulse rounded-xl border border-border bg-muted" }
        }
    }
}

fn day_cycle_label(value: &str) -> &'static str {
    match value {
        "breakfast" => "早餐",
        "lunch" => "午餐",
        "dinner" => "晚餐",
        "snack" => "加餐",
        _ => "餐食",
    }
}
