use dioxus::prelude::*;

#[component]
pub fn AboutPage() -> Element {
    rsx! {
        section { class: "mx-auto flex w-full max-w-4xl flex-col gap-16 py-12 sm:py-20",
            div { class: "text-center",
                svg { class: "mx-auto mb-8 h-12 w-12 text-foreground/40", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "0.75",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 3v18m0-18c-3.314 0-6 2.686-6 6s2.686 6 6 6 6-2.686 6-6-2.686-6-6-6z" }
                }
                h1 { class: "font-serif text-4xl leading-tight text-foreground sm:text-6xl", "为日常生活而生的饮食记忆。" }
                p { class: "mx-auto mt-8 max-w-2xl text-lg leading-relaxed text-muted-foreground",
                    "Warmmy 围绕那些每天重复出现的小决定而设计：你吃了什么、身体感受如何、需要避免什么，以及你的日常正在如何形成。"
                }
            }
            div { class: "mt-12 grid gap-12 sm:grid-cols-3 sm:gap-8",
                AboutFeature {
                    title: "本地优先",
                    body: "你的日常数据从设备本地开始，支持原生移动端与桌面端。",
                    path_d: "M20 12V8H6a2 2 0 01-2-2c0-1.1.9-2 2-2h12v4"
                }
                AboutFeature {
                    title: "以餐食为中心",
                    body: "应用聚焦于食物、营养上下文、个人偏好与每日对话式记忆。",
                    path_d: "M12 21a9 9 0 100-18 9 9 0 000 18zm0 0v-9m0 0L7.5 7.5M12 12l4.5-4.5"
                }
                AboutFeature {
                    title: "陪伴式体验",
                    body: "Warmmy 保持安静、可对话的界面，而不是把饮食变成繁琐管理。",
                    path_d: "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
                }
            }
        }
    }
}

#[component]
fn AboutFeature(title: &'static str, body: &'static str, path_d: &'static str) -> Element {
    rsx! {
        article { class: "flex flex-col items-center text-center",
            svg { class: "mb-6 h-10 w-10 text-foreground/60", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "{path_d}" }
            }
            h2 { class: "font-serif text-2xl text-foreground", "{title}" }
            p { class: "mt-4 text-sm leading-relaxed text-muted-foreground", "{body}" }
        }
    }
}
