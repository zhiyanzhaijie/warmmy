use dioxus::prelude::*;

use crate::components::SEO;

#[component]
pub fn DownloadPage() -> Element {
    rsx! {
        SEO {
            title: "下载",
            description: "下载 Warmmy 最新版本，在 macOS、Windows、iOS 与 Android 设备上使用屋米。",
            keywords: "下载 Warmmy,Warmmy 最新版本,屋米下载,macOS,Windows,iOS,Android",
        }
        section { class: "mx-auto flex w-full max-w-3xl flex-col items-center py-16 text-center sm:py-24",
            svg { class: "mb-10 h-16 w-16 text-foreground/30", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "0.5",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" }
            }
            h1 { class: "font-serif text-5xl leading-tight sm:text-7xl", "获取最新版本" }
            p { class: "mt-8 max-w-xl text-lg leading-relaxed text-muted-foreground",
                "Warmmy 通过 GitHub 发布。请在最新版本中选择与你设备匹配的安装包"
            }
            div { class: "mt-16 w-full border-t border-foreground/10 pt-16",
                h2 { class: "font-serif text-3xl", "最新版本" }
                p { class: "mx-auto mt-4 max-w-md text-base leading-relaxed text-muted-foreground",
                    "macOS、Windows、iOS 与 Android 的安装包会在可用后陆续上传到 GitHub 发布页。"
                }
                a {
                    class: "mt-10 inline-flex h-14 items-center justify-center rounded-full bg-foreground px-10 text-sm font-medium tracking-wide text-background no-underline transition-transform hover:scale-105",
                    href: crate::github_releases_url(),
                    target: "_blank",
                    rel: "noreferrer",
                    "打开 GitHub 发布页"
                }
            }
        }
    }
}
