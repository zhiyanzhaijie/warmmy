use dioxus::prelude::*;

use crate::components::SEO;

#[component]
pub fn AboutPage() -> Element {
    rsx! {
        SEO {
            title: "关于屋米",
            description: "了解 Warmmy 的设计理念：本地优先、以餐食为中心，用安静可对话的方式陪伴日常饮食记录。",
            keywords: "关于 Warmmy,关于屋米,本地优先,饮食伙伴,饮食记录,开源应用",
        }
        section { class: "mx-auto w-full max-w-5xl px-6 py-20 sm:py-32",
            div { class: "flex flex-col items-center text-center border-b border-foreground/10 pb-16 sm:pb-24",
                svg { class: "mb-8 h-10 w-10 text-foreground/30", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1", stroke_linecap: "round", stroke_linejoin: "round",
                    path { d: "M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z" }
                    path { d: "M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12" }
                }
                h1 { class: "font-serif text-5xl leading-[1.15] tracking-wide text-foreground sm:text-6xl md:text-7xl",
                    "为日常饮食而生",
                    br { class: "hidden sm:block" }
                    span { class: "text-muted-foreground", "的屋米小助手" }
                }
                p { class: "mt-10 mb-8 max-w-2xl text-lg leading-relaxed text-muted-foreground sm:text-xl",
                    "Warmmy 围绕那些每天重复出现的小决定而设计：",
                    span { class: "mt-6 block font-serif text-xl leading-relaxed text-foreground/80 sm:text-2xl",
                        "你吃了什么、身体感受如何、需要避免什么，以及你的日常正在如何形成。"
                    }
                }
            }

            div { class: "mt-16 grid gap-12 sm:mt-24 md:grid-cols-3 md:gap-8 lg:gap-12",
                AboutFeature {
                    title: "本地优先",
                    body: "你的日常数据从设备本地开始，支持原生移动端与桌面端。",
                    offset_class: "",
                    svg { class: "h-5 w-5", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                        rect { width: "20", height: "5", x: "2", y: "4", rx: "1" }
                        path { d: "M4 9v9a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9" }
                        path { d: "M10 13h4" }
                    }
                }
                AboutFeature {
                    title: "以餐食为中心",
                    body: "应用聚焦于食物、营养上下文、个人偏好与每日对话式记忆。",
                    offset_class: "md:mt-16",
                    svg { class: "h-5 w-5", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                        path { d: "M3 2v7c0 1.1.9 2 2 2h4a2 2 0 0 0 2-2V2" }
                        path { d: "M7 2v20" }
                        path { d: "M21 15V2v0a5 5 0 0 0-5 5v6c0 1.1.9 2 2 2h3Zm0 0v7" }
                    }
                }
                AboutFeature {
                    title: "陪伴式体验",
                    body: "屋米 保持安静、可对话的界面，而不是把饮食变成繁琐管理。",
                    offset_class: "md:mt-32",
                    svg { class: "h-5 w-5", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                        path { d: "M17 8h1a4 4 0 1 1 0 8h-1" }
                        path { d: "M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z" }
                        line { x1: "6", x2: "6", y1: "2", y2: "4" }
                        line { x1: "10", x2: "10", y1: "2", y2: "4" }
                        line { x1: "14", x2: "14", y1: "2", y2: "4" }
                    }
                }
            }
        }
    }
}

#[component]
fn AboutFeature(
    title: &'static str,
    body: &'static str,
    offset_class: &'static str,
    children: Element,
) -> Element {
    rsx! {
        article { class: "flex flex-col items-start {offset_class}",
            div { class: "mb-6 flex h-12 w-12 items-center justify-center rounded-2xl bg-foreground/5 text-foreground/70",
                {children}
            }
            h2 { class: "font-serif text-xl tracking-wide text-foreground", "{title}" }
            p { class: "mt-3 text-[15px] leading-relaxed text-muted-foreground", "{body}" }
        }
    }
}
