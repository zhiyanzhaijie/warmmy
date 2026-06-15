use dioxus::prelude::*;

use crate::Route;

pub mod theme_switcher;
use theme_switcher::ThemeSwitcher;

#[component]
pub fn SiteLayout() -> Element {
    rsx! {
        div { class: "relative flex min-h-screen flex-col overflow-x-hidden bg-background text-foreground selection:bg-foreground/10",
            svg {
                class: "fixed inset-0 z-0 h-full w-full pointer-events-none text-foreground/[0.02]",
                view_box: "0 0 100 100",
                preserve_aspect_ratio: "none",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "0.2",
                path { d: "M0,20 Q50,40 100,10" }
                path { d: "M0,40 Q60,60 100,30" }
                path { d: "M0,60 Q70,80 100,50" }
                path { d: "M0,80 Q80,100 100,70" }
            }
            SiteNav {}
            main { class: "relative z-10 mx-auto flex w-full max-w-5xl flex-1 flex-col px-6 pb-24 pt-32 sm:px-12",
                Outlet::<Route> {}
            }
            SiteFooter {}
        }
    }
}

#[component]
fn SiteNav() -> Element {
    rsx! {
        header { class: "fixed inset-x-0 top-0 z-50 bg-background py-8",
            nav { class: "mx-auto flex w-full max-w-5xl items-center justify-between px-6 sm:px-12",
                Link { to: Route::HomePage {}, class: "font-serif text-2xl tracking-wide text-foreground no-underline",
                    "Warmmy"
                }
                div { class: "flex items-center gap-8 text-sm font-medium",
                    NavLink { to: Route::HomePage {}, label: "首页" }
                    NavLink { to: Route::GuidePage {}, label: "指南" }
                    NavLink { to: Route::AboutPage {}, label: "关于" }
                    NavLink { to: Route::DownloadPage {}, label: "下载" }
                    ThemeSwitcher {}
                }
            }
        }
    }
}

#[component]
fn NavLink(to: Route, label: &'static str) -> Element {
    rsx! {
        Link {
            to,
            class: "text-muted-foreground no-underline transition-colors hover:text-foreground",
            "{label}"
        }
    }
}

#[component]
fn SiteFooter() -> Element {
    rsx! {
        footer { class: "relative z-10 mx-auto w-full max-w-5xl px-6 pb-12 sm:px-12",
            div { class: "flex flex-col gap-4 border-t border-foreground/10 pt-8 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between",
                p { "Warmmy。记录饮食记忆的安静空间。" }
                a {
                    class: "text-foreground transition-opacity hover:opacity-70",
                    href: crate::github_releases_url(),
                    target: "_blank",
                    rel: "noreferrer",
                    "GitHub 发布页"
                }
            }
        }
    }
}
