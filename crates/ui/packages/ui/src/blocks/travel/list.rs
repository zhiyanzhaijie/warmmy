use api::meal;
use dioxus::prelude::*;
use dioxus_icons::lucide::{CalendarDays, CircleGauge, Tent, TrendingUp};
use dioxus_primitives::{ContentAlign, ContentSide};

use crate::blocks::CurrentUserContext;
use crate::components::ui::popover::{PopoverContent, PopoverRoot, PopoverTrigger};

use super::metrics::{compact_number, date_label, parse_metrics};

#[derive(Clone, PartialEq)]
struct ContourPath {
    d: String,
    stroke: &'static str,
    width: &'static str,
}

#[derive(Clone, Copy)]
struct MountainSpec {
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    seed: f32,
}

#[derive(Clone, Copy)]
struct MountainPlacement {
    center_x: f32,
    center_y: f32,
    outer_radius_y: f32,
}

#[derive(Clone, PartialEq)]
struct DisplaySummary {
    summary: meal::MealDaySummaryDTO,
    display_score: f32,
}

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
    let display_summaries = build_display_summaries(&summaries);
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
        div { class: "relative h-full min-h-0 overflow-y-auto px-4 py-4 pb-28 md:px-8 md:py-8 md:pb-12",
            div { class: "relative mx-auto w-full max-w-6xl",
                section { class: "min-h-0 pr-16 md:pr-28",
                    if !error().is_empty() {
                        div { class: "rounded-xl border border-border bg-card px-4 py-3 text-sm text-destructive",
                            "{error}"
                        }
                    } else if loading() {
                        TimelineSkeleton {}
                    } else if summaries.is_empty() {
                        EmptyTimeline {}
                    } else {
                        div {
                            class: "relative pb-4 pl-1 pr-1 md:pr-4",
                            style: "min-height: {map_min_height(&display_summaries)}rem;",
                            MapTexture { summaries: display_summaries.clone() }
                            RoutePath { summaries: display_summaries.clone() }
                            for (index, display) in display_summaries.iter().cloned().enumerate() {
                                TimelineItem {
                                    display,
                                    index,
                                    summaries: display_summaries.clone(),
                                    featured: index == 0,
                                }
                            }
                        }
                    }
                }

                aside { class: "pointer-events-none fixed right-4 top-4 z-20 md:right-8 md:top-8",
                    div { class: "flex flex-col items-end gap-3 py-1",
                        h1 { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground [writing-mode:vertical-rl]",
                            "Diet travel"
                        }
                        div { class: "flex flex-col items-end gap-2",
                            StatTile { label: "天数".to_string(), value: count.to_string(), icon: "calendar".to_string() }
                            StatTile { label: "平均".to_string(), value: compact_number(average_score), icon: "trend".to_string() }
                            StatTile { label: "最佳".to_string(), value: compact_number(best_score), icon: "score".to_string() }
                        }
                    }
                }
            }
        }
    }
}

fn build_display_summaries(summaries: &[meal::MealDaySummaryDTO]) -> Vec<DisplaySummary> {
    summaries
        .iter()
        .cloned()
        .map(|summary| {
            let display_score = summary.overall_score;
            DisplaySummary {
                summary,
                display_score,
            }
        })
        .collect()
}

#[component]
fn StatTile(label: String, value: String, icon: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-end gap-1.5 text-right",
            div { class: "order-2 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border border-border bg-background/70 text-muted-foreground",
                if icon == "calendar" {
                    CalendarDays { size: 11 }
                } else if icon == "trend" {
                    TrendingUp { size: 11 }
                } else {
                    CircleGauge { size: 11 }
                }
            }
            div { class: "min-w-0",
                p { class: "text-[9px] leading-none text-muted-foreground", "{label}" }
                p { class: "truncate text-sm font-semibold leading-tight tracking-[-0.2px] text-foreground", "{value}" }
            }
        }
    }
}

#[component]
fn MapTexture(summaries: Vec<DisplaySummary>) -> Element {
    let mut contours = Vec::new();
    for (index, display) in summaries.iter().enumerate() {
        contours.extend(build_mountain_contours(
            display.display_score,
            mountain_spec_for_index(index, &summaries),
        ));
    }
    let view_box = format!("0 0 100 {:.2}", map_view_height(&summaries));

    rsx! {
        svg {
            class: "pointer-events-none absolute inset-0 h-full w-full overflow-visible opacity-100",
            view_box,
            preserve_aspect_ratio: "none",
            for contour in contours {
                path {
                    d: "{contour.d}",
                    fill: "none",
                    stroke: "{contour.stroke}",
                    stroke_width: "{contour.width}",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
            }
        }
    }
}

fn mountain_spec_for_index(index: usize, summaries: &[DisplaySummary]) -> MountainSpec {
    let placement = mountain_placements(summaries)
        .get(index)
        .copied()
        .unwrap_or(MountainPlacement {
            center_x: 25.0,
            center_y: 24.0,
            outer_radius_y: 12.0,
        });
    let cell_width = 100.0 / 6.0;

    MountainSpec {
        center_x: placement.center_x,
        center_y: placement.center_y,
        radius_x: cell_width * 0.28,
        radius_y: placement.outer_radius_y / contour_scale(max_ring_index(
            summaries
                .get(index)
                .map(|display| display.display_score)
                .unwrap_or(0.0),
        )),
        seed: 2.4 + index as f32 * 1.73,
    }
}

fn mountain_placements(summaries: &[DisplaySummary]) -> Vec<MountainPlacement> {
    let mut placements: Vec<MountainPlacement> = Vec::with_capacity(summaries.len());
    let mut center_y = 24.0;
    let gap = 12.0;

    for (index, display) in summaries.iter().enumerate() {
        let outer_radius_y = mountain_outer_radius_y(display.display_score);
        if let Some(previous) = placements.last() {
            center_y = previous.center_y + previous.outer_radius_y + outer_radius_y + gap;
        }
        let col = if index == 0 {
            2.0
        } else if index % 2 == 1 {
            5.0
        } else {
            2.0
        };
        placements.push(MountainPlacement {
            center_x: ((col - 0.5) / 6.0) * 100.0,
            center_y,
            outer_radius_y,
        });
    }

    placements
}

fn mountain_outer_radius_y(score: f32) -> f32 {
    8.0 + max_ring_index(score) as f32 * 1.7
}

fn max_ring_index(score: f32) -> usize {
    contour_ring_count(score).saturating_sub(1)
}

fn map_view_height(summaries: &[DisplaySummary]) -> f32 {
    let placements = mountain_placements(summaries);
    let Some(last) = placements.last() else {
        return 100.0;
    };
    (last.center_y + last.outer_radius_y + 18.0).max(100.0)
}

fn map_min_height(summaries: &[DisplaySummary]) -> f32 {
    (map_view_height(summaries) / 100.0 * 56.0).max(56.0)
}

fn build_mountain_contours(score: f32, spec: MountainSpec) -> Vec<ContourPath> {
    let rings = contour_ring_count(score);
    let base_points = irregular_base_ring_points(spec);
    (0..rings)
        .map(|ring| {
            let scale = contour_scale(ring);
            let points = scaled_ring_points(&base_points, spec, ring, scale);
            ContourPath {
                d: closed_curve_path(&points),
                stroke: contour_stroke(ring),
                width: contour_width(ring),
            }
        })
        .collect()
}

fn contour_ring_count(score: f32) -> usize {
    let score = score.clamp(0.0, 100.0);
    let base = if score < 20.0 {
        1
    } else if score < 50.0 {
        2
    } else if score < 70.0 {
        3
    } else if score < 90.0 {
        4
    } else {
        4 + ((score - 90.0) / 5.0).floor() as usize
    };
    base * 2
}

fn contour_scale(ring: usize) -> f32 {
    let mut scale = 1.0;
    for step in 1..=ring {
        let factor = contour_scale_factor(step);
        scale *= factor;
    }
    scale
}

fn contour_scale_factor(step: usize) -> f32 {
    match step {
        1 => 2.00,
        2 => 1.55,
        3 => 1.42,
        4 => 1.34,
        5 => 1.29,
        6 => 1.25,
        7 => 1.23,
        8 => 1.21,
        _ => 1.20,
    }
}

fn irregular_base_ring_points(spec: MountainSpec) -> Vec<(f32, f32)> {
    const POINTS: usize = 9;
    (0..POINTS)
        .map(|index| {
            let t = index as f32 / POINTS as f32;
            let angle = t * std::f32::consts::TAU;
            let noise = contour_noise(spec.seed, index);
            let skew = spec.seed.sin() * 0.22;
            let rx = spec.radius_x * (1.0 + noise * 0.30);
            let ry = spec.radius_y * (1.0 - noise * 0.22);
            let x = spec.center_x + angle.cos() * rx + angle.sin() * skew;
            let y = spec.center_y + angle.sin() * ry + angle.cos() * skew * 0.6;
            (x, y)
        })
        .collect()
}

fn scaled_ring_points(
    base_points: &[(f32, f32)],
    spec: MountainSpec,
    ring: usize,
    scale: f32,
) -> Vec<(f32, f32)> {
    let drift_x = (spec.seed + ring as f32 * 0.71).sin() * ring as f32 * 0.14;
    let drift_y = (spec.seed * 0.7 + ring as f32 * 0.53).cos() * ring as f32 * 0.11;
    base_points
        .iter()
        .map(|(x, y)| {
            (
                spec.center_x + (x - spec.center_x) * scale + drift_x,
                spec.center_y + (y - spec.center_y) * scale + drift_y,
            )
        })
        .collect()
}

fn contour_noise(seed: f32, index: usize) -> f32 {
    let n = seed * 1.618 + index as f32 * 0.917;
    (n.sin() * 0.65) + ((n * 1.91).cos() * 0.35)
}

fn closed_curve_path(points: &[(f32, f32)]) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut d = format!("M {:.2} {:.2}", points[0].0, points[0].1);
    for index in 0..points.len() {
        let current = points[index];
        let next = points[(index + 1) % points.len()];
        let previous = points[(index + points.len() - 1) % points.len()];
        let after_next = points[(index + 2) % points.len()];
        let c1 = (
            current.0 + (next.0 - previous.0) * 0.16,
            current.1 + (next.1 - previous.1) * 0.16,
        );
        let c2 = (
            next.0 - (after_next.0 - current.0) * 0.16,
            next.1 - (after_next.1 - current.1) * 0.16,
        );
        d.push_str(&format!(
            " C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2}",
            c1.0, c1.1, c2.0, c2.1, next.0, next.1
        ));
    }
    d.push_str(" Z");
    d
}

fn contour_stroke(ring: usize) -> &'static str {
    match ring % 4 {
        1 => "rgba(28,28,28,0.118)",
        2 => "rgba(15,122,77,0.140)",
        3 => "rgba(255,173,26,0.160)",
        _ => "rgba(28,28,28,0.084)",
    }
}

fn contour_width(ring: usize) -> &'static str {
    match ring {
        0 => "0.15",
        1 => "0.17",
        2 => "0.20",
        _ => "0.23",
    }
}

#[component]
fn RoutePath(summaries: Vec<DisplaySummary>) -> Element {
    let height = map_view_height(&summaries);
    let route = route_path_for_mountains(&summaries);
    let view_box = format!("0 0 100 {height:.2}");

    rsx! {
        svg {
            class: "pointer-events-none absolute left-0 top-0 h-full w-full overflow-visible",
            view_box,
            preserve_aspect_ratio: "none",
            path {
                d: "{route}",
                fill: "none",
                stroke: "rgba(28,28,28,0.20)",
                stroke_width: "0.5",
                stroke_linecap: "round",
                stroke_dasharray: "1.2 1.6",
            }
            path {
                d: "{route}",
                fill: "none",
                stroke: "rgba(15,122,77,0.16)",
                stroke_width: "0.18",
                stroke_linecap: "round",
            }
        }
    }
}

fn route_path_for_mountains(summaries: &[DisplaySummary]) -> String {
    let points = (0..summaries.len())
        .map(|index| {
            let spec = mountain_spec_for_index(index, summaries);
            (spec.center_x, spec.center_y)
        })
        .collect::<Vec<_>>();

    if points.is_empty() {
        return String::new();
    }

    if points.len() == 1 {
        return format!("M {:.2} {:.2}", points[0].0, points[0].1);
    }

    let mut d = format!("M {:.2} {:.2}", points[0].0, points[0].1);
    for index in 0..points.len() - 1 {
        let current = points[index];
        let next = points[index + 1];
        let previous = if index == 0 { current } else { points[index - 1] };
        let after_next = if index + 2 >= points.len() {
            next
        } else {
            points[index + 2]
        };
        let c1 = (
            current.0 + (next.0 - previous.0) * 0.18,
            current.1 + (next.1 - previous.1) * 0.18,
        );
        let c2 = (
            next.0 - (after_next.0 - current.0) * 0.18,
            next.1 - (after_next.1 - current.1) * 0.18,
        );
        d.push_str(&format!(
            " C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2}",
            c1.0, c1.1, c2.0, c2.1, next.0, next.1
        ));
    }
    d
}

#[component]
fn TimelineItem(
    display: DisplaySummary,
    index: usize,
    summaries: Vec<DisplaySummary>,
    featured: bool,
) -> Element {
    let mut expanded = use_signal(|| featured);
    let summary = display.summary;
    let metrics = parse_metrics(&summary);
    let score = compact_number(display.display_score);
    let nutrition_score = compact_number(summary.nutrition_score);
    let expectation_score = compact_number(summary.expectation_match_score);
    let calories = compact_number(metrics.total_nutrition.calories);
    let spec = mountain_spec_for_index(index, &summaries);
    let left = format!("{:.2}%", spec.center_x);
    let top = format!("{:.2}%", spec.center_y / map_view_height(&summaries) * 100.0);
    let side = if spec.center_x > 50.0 {
        ContentSide::Left
    } else {
        ContentSide::Right
    };

    rsx! {
        div {
            class: "absolute z-10",
            style: "left: {left}; top: {top};",
            div {
                class: "absolute left-1/2 top-0 -translate-x-1/2 -translate-y-full",
                onmouseenter: move |_| expanded.set(true),
                onmouseleave: move |_| {
                    if !featured {
                        expanded.set(false);
                    }
                },
                PopoverRoot {
                    open: expanded(),
                    on_open_change: move |open| {
                        expanded.set(open);
                    },
                    PopoverTrigger {
                        class: "!border-0 !bg-transparent !p-0 !shadow-none !outline-none flex flex-col items-center gap-1 text-foreground",
                        div { class: if featured { "text-foreground" } else { "text-muted-foreground" },
                            Tent { size: if featured { 44 } else { 38 } }
                        }
                        span { class: "rounded-full border border-border bg-background/90 px-2.5 py-0.5 text-[10px] font-semibold text-foreground",
                            "{date_label(&summary.session_id)}"
                        }
                    }
                    PopoverContent {
                        side,
                        align: ContentAlign::Center,
                        class: "w-[min(24rem,calc(100vw-2rem))] max-h-[min(30rem,calc(100vh-2rem))] overflow-auto border border-border bg-card/95 p-0 text-left shadow-sm".to_string(),
                        Link {
                            to: format!("/travel/{}", summary.session_id),
                            class: "block w-full rounded-xl p-4 transition-opacity active:opacity-80 md:p-5",
                            article {
                                div { class: "flex items-start justify-between gap-4",
                                    div { class: "min-w-0",
                                        p { class: "text-[10px] font-semibold uppercase tracking-[0.22em] text-muted-foreground",
                                            if featured { "Latest stop" } else { "Map stop" }
                                        }
                                        p { class: "mt-3 line-clamp-5 text-base leading-snug text-foreground md:text-lg",
                                            "{summary.content}"
                                        }
                                    }
                                    div { class: "shrink-0 text-right",
                                        p { class: "text-[10px] text-muted-foreground", "score" }
                                        p { class: if featured { "text-5xl font-semibold leading-none tracking-[-1px] text-foreground" } else { "text-4xl font-semibold leading-none tracking-[-0.8px] text-foreground" },
                                            "{score}"
                                        }
                                    }
                                }
                                div { class: "mt-5 grid grid-cols-2 gap-2 text-[11px] md:grid-cols-4",
                                    TravelMetric { label: "meals".to_string(), value: metrics.meal_count.to_string() }
                                    TravelMetric { label: "营养".to_string(), value: nutrition_score }
                                    TravelMetric { label: "期望".to_string(), value: expectation_score }
                                    TravelMetric { label: "热量".to_string(), value: calories }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TravelMetric(label: String, value: String) -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-background/70 px-3 py-2",
            p { class: "text-[10px] text-muted-foreground", "{label}" }
            p { class: "mt-1 text-sm font-semibold leading-none text-foreground", "{value}" }
        }
    }
}

#[component]
fn TimelineSkeleton() -> Element {
    rsx! {
        div { class: "flex flex-col gap-3",
            for _ in 0..4 {
                div { class: "h-24 animate-pulse rounded-xl border border-border bg-muted" }
            }
        }
    }
}

#[component]
fn EmptyTimeline() -> Element {
    rsx! {
        div { class: "rounded-2xl border border-dashed border-border bg-card/70 px-5 py-12 text-center",
            CalendarDays { size: 32, class: "mx-auto text-muted-foreground" }
            h2 { class: "mt-4 text-3xl font-semibold tracking-[-0.6px] text-foreground", "还没有旅程记录" }
            p { class: "mx-auto mt-2 max-w-md text-sm leading-relaxed text-muted-foreground",
                "在 chat 中完成某一天的餐食敲定和 summary 后，它会出现在这里。"
            }
        }
    }
}
