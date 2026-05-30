use crate::blocks::{
    ChatBlock, ConversationTransitionContext, PendingConversationMessage, ACTIVE_SESSION_ID,
    CHAT_INPUT, CHAT_MESSAGES, CHAT_NEXT_ID,
};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::textarea::{Textarea, TextareaVariant};
use crate::today_session_id;
use chrono::{Datelike, Local};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Send, Sparkles};

#[component]
pub fn HomeView() -> Element {
    let mut input = use_signal(String::new);
    let mut transition = use_context::<ConversationTransitionContext>();
    let today_index = Local::now().weekday().num_days_from_monday() as usize;

    let mut start_chat_with_msg = move || {
        let content = input().trim().to_string();
        if content.is_empty() {
            return;
        }

        input.set(String::new());
        let session_id = today_session_id();

        *ACTIVE_SESSION_ID.write() = Some(session_id.clone());
        CHAT_MESSAGES.write().clear();
        CHAT_INPUT.write().clear();
        *CHAT_NEXT_ID.write() = 1;

        transition.pending.set(Some(PendingConversationMessage {
            session_id,
            content,
            started: false,
        }));
    };

    if (transition.pending)().is_some() {
        return rsx! {
            main {
                class: "h-full min-h-0 overflow-hidden",
                ChatBlock { session_id: None }
            }
        };
    }

    rsx! {
        div { class: "relative h-full min-h-0 overflow-hidden",
            div {
                class: "pointer-events-none absolute inset-0",
                style: "background:
                    radial-gradient(circle at 18% 14%, rgba(255, 173, 26, 0.16), transparent 22rem),
                    radial-gradient(circle at 82% 18%, rgba(15, 122, 77, 0.10), transparent 20rem),
                    linear-gradient(180deg, transparent 0%, color-mix(in oklab, var(--background) 86%, transparent) 62%, var(--background) 100%);",
            }
            PizzaWeekBackground { active_index: today_index }
            div { class: "relative flex h-full min-h-0 flex-col justify-end px-4 pb-5 md:px-10 md:pb-10",
                div { class: "mx-auto flex w-full max-w-3xl flex-col gap-5",
                    div { class: "max-w-2xl",
                        div { class: "mb-4 flex h-12 w-12 items-center justify-center rounded-[1.25rem] border border-border bg-card/75 text-foreground backdrop-blur md:h-14 md:w-14",
                            Sparkles { size: 24 }
                        }
                        p { class: "text-[11px] font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Warmmy local-first" }
                        h1 { class: "mt-3 font-doodle text-4xl font-semibold leading-none text-foreground md:text-6xl",
                            "Eat with memory"
                        }
                        p { class: "mt-4 max-w-xl text-sm leading-relaxed text-muted-foreground md:text-base",
                            "记录餐食，或直接问下一顿吃什么。"
                        }
                    }

                    div { class: "w-full rounded-[2rem] border border-border bg-card/80 p-2 shadow-lg backdrop-blur",
                        div { class: "flex items-end gap-2 rounded-[1.5rem] border border-border bg-background/95 p-2 focus-within:shadow-md",
                            Textarea {
                                variant: TextareaVariant::Ghost,
                                class: "max-h-40 min-h-12 min-w-0 flex-1 resize-none overflow-y-auto border-none bg-transparent px-4 py-3 text-base font-medium leading-relaxed text-foreground shadow-none outline-none placeholder:text-muted-foreground placeholder:whitespace-nowrap placeholder:overflow-hidden placeholder:text-ellipsis [field-sizing:content]",
                                rows: "1",
                                placeholder: "记录餐食，或询问下一顿吃什么...",
                                value: input(),
                                oninput: move |e: FormEvent| {
                                    input.set(e.value());
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter && !e.modifiers().shift() {
                                        start_chat_with_msg();
                                    }
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Icon,
                                class: "mb-1 rounded-full bg-foreground p-3 text-background shadow-sm hover:opacity-90",
                                onclick: move |_| start_chat_with_msg(),
                                Send { size: 20, class: "ml-0.5" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PizzaWeekBackground(active_index: usize) -> Element {
    let slices = pizza_slices(active_index);

    rsx! {
        svg {
            class: "pointer-events-none absolute left-1/2 top-[8%] h-[min(82vw,560px)] w-[min(82vw,560px)] -translate-x-1/2 text-foreground opacity-70 md:top-[6%]",
            view_box: "0 0 520 520",
            role: "presentation",
            "aria-hidden": "true",
            g { transform: "translate(260 260)",
                path {
                    d: "M -190 -20 C -186 -94 -124 -168 -36 -190 C 60 -212 154 -166 192 -78 C 234 20 194 132 98 184 C 12 230 -104 208 -162 132 C -192 92 -204 38 -190 -20 Z",
                    fill: "rgba(209, 129, 45, 0.08)",
                    stroke: "#8f5f2f",
                    stroke_width: "8",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    opacity: "0.24",
                }
                path {
                    d: "M -184 -6 C -178 -86 -116 -156 -28 -178 C 58 -198 144 -154 180 -72 C 218 20 180 120 92 170 C 8 214 -92 194 -148 122 C -178 82 -192 36 -184 -6 Z",
                    fill: "none",
                    stroke: "#bd8343",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    opacity: "0.38",
                }
                for slice in slices {
                    g {
                        key: "{slice.index}",
                        path {
                            d: slice.path,
                            fill: slice.fill,
                            stroke: slice.stroke,
                            stroke_width: "2.6",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            opacity: slice.opacity,
                        }
                        path {
                            d: slice.shadow_path,
                            fill: "none",
                            stroke: "#8f5f2f",
                            stroke_width: "2.2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            opacity: slice.shadow_opacity,
                        }
                    }
                }
                PizzaTopping { x: -96.0, y: -92.0, kind: "tomato", active: active_index == 0, opacity: topping_opacity(active_index, 0) }
                PizzaTopping { x: -34.0, y: -126.0, kind: "green", active: active_index == 1, opacity: topping_opacity(active_index, 1) }
                PizzaTopping { x: 78.0, y: -112.0, kind: "tomato", active: active_index == 2, opacity: topping_opacity(active_index, 2) }
                PizzaTopping { x: 118.0, y: -18.0, kind: "crumb", active: active_index == 3, opacity: topping_opacity(active_index, 3) }
                PizzaTopping { x: 54.0, y: 92.0, kind: "tomato", active: active_index == 4, opacity: topping_opacity(active_index, 4) }
                PizzaTopping { x: -62.0, y: 112.0, kind: "green", active: active_index == 5, opacity: topping_opacity(active_index, 5) }
                PizzaTopping { x: -128.0, y: 18.0, kind: "crumb", active: active_index == 6, opacity: topping_opacity(active_index, 6) }
                path {
                    d: "M -108 42 C -76 18, -44 22, -14 48 C 20 78, 58 78, 92 46",
                    fill: "none",
                    stroke: "#bd8343",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    opacity: "0.22",
                }
            }
        }
    }
}

#[derive(Clone)]
struct PizzaSlice {
    index: usize,
    path: String,
    shadow_path: String,
    fill: &'static str,
    stroke: &'static str,
    opacity: &'static str,
    shadow_opacity: &'static str,
}

fn pizza_slices(active_index: usize) -> Vec<PizzaSlice> {
    (0..7)
        .map(|index| {
            let base_start = -90.0 + index as f64 * 360.0 / 7.0;
            let base_end = -90.0 + (index + 1) as f64 * 360.0 / 7.0;
            let start = base_start + 2.4 + wobble(index) * 0.5;
            let end = base_end - 2.4 + wobble(index + 3) * 0.5;
            let radius = 176.0 + wobble(index) * 5.0;
            let is_active = index == active_index;
            let distance = index.abs_diff(active_index);
            PizzaSlice {
                index,
                path: sector_path(start, end, radius, index as f64),
                shadow_path: sector_path(start + 1.1, end + 1.1, radius + 10.0, index as f64 + 2.0),
                fill: if is_active {
                    "rgba(244, 176, 75, 0.28)"
                } else {
                    "rgba(209, 129, 45, 0.08)"
                },
                stroke: if is_active { "#b86b10" } else { "#bd8343" },
                opacity: if is_active {
                    "0.92"
                } else if distance == 1 || distance == 6 {
                    "0.36"
                } else {
                    "0.18"
                },
                shadow_opacity: if is_active { "0.36" } else { "0.14" },
            }
        })
        .collect()
}

fn sector_path(start_deg: f64, end_deg: f64, radius: f64, wobble_seed: f64) -> String {
    let (sx, sy) = angle_to_point(start_deg, radius);
    let (ex, ey) = angle_to_point(end_deg, radius);
    let mid = (start_deg + end_deg) / 2.0;
    let (mx, my) = angle_to_point(mid, radius + wobble_seed.sin() * 7.0);
    let center_x = wobble_seed.sin() * 3.0;
    let center_y = wobble_seed.cos() * 3.0;
    format!("M {center_x:.2} {center_y:.2} L {sx:.2} {sy:.2} Q {mx:.2} {my:.2} {ex:.2} {ey:.2} Z")
}

fn angle_to_point(deg: f64, radius: f64) -> (f64, f64) {
    let radians = deg.to_radians();
    (radians.cos() * radius, radians.sin() * radius)
}

#[component]
fn PizzaTopping(
    x: f64,
    y: f64,
    kind: &'static str,
    active: bool,
    opacity: &'static str,
) -> Element {
    let tomato = if active { "#d64c32" } else { "#a95a46" };
    let green = if active { "#2f7d4f" } else { "#4f765b" };
    let crumb = if active { "#7b4b2a" } else { "#8d6b4a" };
    rsx! {
        if kind == "tomato" {
            circle {
                cx: "{x}",
                cy: "{y}",
                r: "13",
                fill: "rgba(214, 76, 50, 0.10)",
                stroke: tomato,
                stroke_width: "3",
                opacity,
            }
            circle {
                cx: "{x + 4.0}",
                cy: "{y - 3.0}",
                r: "3",
                fill: tomato,
                opacity,
            }
        } else if kind == "green" {
            path {
                d: format!("M {:.1} {:.1} C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1} C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}", x - 15.0, y + 5.0, x - 9.0, y - 14.0, x + 9.0, y - 14.0, x + 15.0, y + 4.0, x + 5.0, y + 11.0, x - 6.0, y + 11.0, x - 15.0, y + 5.0),
                fill: "rgba(47, 125, 79, 0.10)",
                stroke: green,
                stroke_width: "3",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                opacity,
            }
            path {
                d: format!("M {:.1} {:.1} C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}", x - 8.0, y + 4.0, x - 2.0, y - 2.0, x + 4.0, y - 2.0, x + 9.0, y + 3.0),
                fill: "none",
                stroke: green,
                stroke_width: "2",
                stroke_linecap: "round",
                opacity,
            }
        } else {
            path {
                d: format!("M {:.1} {:.1} l 14 -4 l 8 10 l -10 11 l -13 -5 Z", x - 9.0, y - 5.0),
                fill: "rgba(123, 75, 42, 0.10)",
                stroke: crumb,
                stroke_width: "3",
                stroke_linejoin: "round",
                opacity,
            }
        }
    }
}

fn topping_opacity(active_index: usize, index: usize) -> &'static str {
    let distance = index.abs_diff(active_index);
    if index == active_index {
        "0.84"
    } else if distance == 1 || distance == 6 {
        "0.40"
    } else {
        "0.22"
    }
}

fn wobble(index: usize) -> f64 {
    match index % 7 {
        0 => -0.7,
        1 => 0.45,
        2 => -0.25,
        3 => 0.8,
        4 => -0.45,
        5 => 0.35,
        _ => -0.15,
    }
}
