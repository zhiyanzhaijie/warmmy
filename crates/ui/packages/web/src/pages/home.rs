use dioxus::prelude::*;

use crate::Route;

const HUMAN_WARMMY: Asset = asset!("/assets/human-warmmy.svg");

#[component]
pub fn HomePage() -> Element {
    rsx! {
        section { class: "grid min-h-[70vh] items-center gap-16 py-12 lg:grid-cols-[1fr_0.8fr] lg:gap-24",
            div { class: "flex flex-col items-start",
                h1 { class: "font-serif text-4xl leading-[1.1] text-foreground sm:text-6xl lg:text-4xl tracking-tight",
                    "呼叫人类，这里是屋米Warmmy"
                }
                p { class: "mt-8 max-w-xl text-lg leading-relaxed text-muted-foreground",
                    "温暖的日常饮食伙伴，活在电子空间的饭搭子。开源应用，对话完全本地，隐私和开销由你自行掌控。"
                }
                div { class: "mt-12 flex flex-col gap-6 sm:flex-row sm:items-center",
                    Link { to: Route::DownloadPage {}, class: "inline-flex h-12 items-center justify-center rounded-full bg-foreground px-8 text-sm font-medium text-background no-underline transition-transform hover:scale-105",
                        "下载 Warmmy"
                    }
                    Link { to: Route::AboutPage {}, class: "group inline-flex items-center gap-2 text-sm font-medium text-foreground no-underline",
                        span { class: "relative",
                            "了解屋米"
                            span { class: "absolute -bottom-1 left-0 h-[1px] w-full origin-left scale-x-0 bg-foreground transition-transform duration-300 group-hover:scale-x-100" }
                        }
                        svg { class: "h-4 w-4 transition-transform group-hover:translate-x-1", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1.5",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M17 8l4 4m0 0l-4 4m4-4H3" }
                        }
                    }
                }
            }
            div { class: "relative mx-auto flex w-full max-w-lg items-center justify-center lg:max-w-none",
                svg {
                    class: "absolute h-[140%] w-[140%] animate-[spin_60s_linear_infinite] text-foreground/10 pointer-events-none",
                    view_box: "0 0 200 200",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "0.5",
                    circle { cx: "100", cy: "100", r: "60", stroke_dasharray: "4 4" }
                    circle { cx: "100", cy: "100", r: "80", stroke_dasharray: "8 8" }
                    circle { cx: "100", cy: "100", r: "100" }
                }
                img {
                    src: HUMAN_WARMMY,
                    class: "relative z-10 w-full object-contain opacity-90 drop-shadow-2xl scale-180 rotate-[24deg] -translate-x-6 sm:-translate-x-10 lg:-translate-x-10",
                    alt: "Warmmy 伙伴插画",
                }
            }
        }
    }
}
