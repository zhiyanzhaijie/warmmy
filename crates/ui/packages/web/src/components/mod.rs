use dioxus::prelude::*;
use dioxus_icons::lucide::Heart;

use crate::Route;

mod seo;
pub mod theme_switcher;
pub use seo::SEO;
use theme_switcher::ThemeSwitcher;

const WARMMY_LOGO: Asset = asset!("/assets/warmmy_logo.svg");

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
        header { class: "fixed inset-x-0 top-0 z-50 bg-background py-5 sm:py-8",
            nav { class: "mx-auto flex w-full max-w-5xl items-center justify-between gap-2 px-3 sm:px-12",
                Link { to: Route::HomePage {}, class: "inline-flex min-w-0 items-center gap-1.5 font-mono text-xl tracking-wide text-foreground no-underline sm:gap-2 sm:text-2xl",
                    img {
                        src: WARMMY_LOGO,
                        alt: "Warmmy logo",
                        class: "h-7 w-7 shrink-0 sm:h-8 sm:w-8",
                    }
                    span { class: "hidden min-[360px]:inline truncate", "Warmmy" }
                }
                div { class: "flex shrink-0 items-center gap-2 text-xs font-medium sm:gap-8 sm:text-sm",
                    NavLink { to: Route::HomePage {}, label: "首页" }
                    NavLink { to: Route::GuidePage {}, label: "指南" }
                    NavLink { to: Route::AboutPage {}, label: "关于" }
                    NavLink { to: Route::DownloadPage {}, label: "下载" }
                    div { class: "mx-1 h-4 w-px bg-border/40 hidden sm:block" }
                    div { class: "flex items-center gap-1 sm:gap-2",
                        ThemeSwitcher {}
                        a {
                        href: "https://github.com/zhiyanzhaijie/warmmy",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "inline-flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground hover:cursor-pointer",
                        svg {
                            class: "lucide lucide-github",
                            fill: "none",
                            height: "18",
                            stroke: "currentColor",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            view_box: "0 0 24 24",
                            width: "18",
                            xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" }
                            path { d: "M9 18c-4.51 2-5-2-7-2" }
                        }
                    }
                    }
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
            div { class: "mt-6 flex flex-wrap items-center justify-center gap-1.5 text-[10px] sm:text-[11px] font-mono text-muted-foreground/40",
                span { "Made with" }
                a {
                    class: "inline-flex items-center gap-1 text-foreground/50 underline-offset-2 transition-colors hover:text-foreground hover:underline",
                    href: "https://dioxuslabs.com/",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    svg {
                        width: "12",
                        height: "12",
                        class: "shrink-0",
                        view_box: "0 0 32 32",
                        fill: "none",
                        g {
                            transform: "translate(-16, -45)",
                            path {
                                d: "m 33.01563,46.549479 c 0,3.07669 -0.8509,5.481687 -2.21485,7.376727 -1.36394,1.89503 -3.3201,3.55681 -5.44726,5.33007 -2.12717,1.77326 -4.42516,3.65721 -6.25196,6.19532 C 17.27476,67.989706 16,70.966946 16,74.983296 h 4.69922 c 0,-3.07668 0.85285,-5.37671 2.2168,-7.27175 1.36395,-1.89504 3.3201,-3.55682 5.44726,-5.33008 2.12717,-1.77326 4.42516,-3.6572 6.25196,-6.19531 1.8268,-2.53811 3.10156,-5.620317 3.10156,-9.636677 z",
                                fill: "#e96020",
                            }
                            path {
                                d: "m 20.388672,70.693359 c -0.635433,-2.16e-4 -1.020686,0.514958 -1.02047,1.150391 -2.16e-4,0.635433 0.385037,1.150607 1.02047,1.150391 h 12.939453 c 0.635433,2.16e-4 1.150607,-0.514958 1.150391,-1.150391 2.16e-4,-0.635433 -0.514958,-1.150607 -1.150391,-1.150391 z",
                                fill: "#2d323b",
                            }
                            path {
                                d: "m 21.75,66.617187 c -0.607971,2.15e-4 -0.954312,0.557976 -0.953448,1.165947 2.16e-4,0.607208 0.34624,1.035009 0.953448,1.035225 h 10.216797 c 0.607208,-2.16e-4 1.099393,-0.492401 1.099609,-1.099609 8.64e-4,-0.607971 -0.491638,-1.101348 -1.099609,-1.101563 z",
                                fill: "#2d323b",
                            }
                            path {
                                d: "m 21.75,53.023437 c -0.607971,2.15e-4 -1.100473,0.493592 -1.099609,1.101563 2.16e-4,0.607208 0.492401,1.099393 1.099609,1.099609 h 10.216797 c 0.607208,-2.16e-4 0.953232,-0.597962 0.953448,-1.20517 8.64e-4,-0.607971 -0.345477,-0.995787 -0.953448,-0.996002 z",
                                fill: "#2d323b",
                            }
                            path {
                                d: "m 20.388672,48.787109 c -0.634671,-2.16e-4 -1.149529,0.513768 -1.150391,1.148438 -2.16e-4,0.635432 0.514959,1.150606 1.150391,1.15039 h 12.939453 c 0.635432,2.16e-4 1.020686,-0.497733 1.02047,-1.133165 -8.62e-4,-0.63467 -0.385799,-1.165879 -1.02047,-1.165663 z",
                                fill: "#2d323b",
                            }
                            path {
                                d: "m 16,46.549479 c 0,4.01636 1.27476,7.098567 3.10156,9.636677 1.8268,2.53811 4.12479,4.42205 6.25196,6.19531 2.12716,1.77326 4.08332,3.65601 5.44726,5.55105 1.36395,1.89504 2.21485,3.9741 2.21485,7.05078 h 4.70117 c 0,-4.01635 -1.27476,-7.03779 -3.10156,-9.5759 -1.8268,-2.53811 -4.12479,-4.42206 -6.25196,-6.19532 -2.12716,-1.77326 -4.08331,-3.43504 -5.44726,-5.33007 -1.36395,-1.89504 -2.2168,-4.255837 -2.2168,-7.332527 z",
                                fill: "#00a8d6",
                            }
                        }
                    }
                    span { "Dioxus" }
                }
                span { "and" }
                Heart {
                    class: "shrink-0 text-red-500",
                    fill: "currentColor",
                    size: 12,
                }
                span { "by" }
                a {
                    class: "text-foreground/50 underline-offset-2 transition-colors hover:text-foreground hover:underline",
                    href: "https://zhiyanzhaijie.space/about",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "zhiyanzhaijie"
                }
                span { "for Rustacean" }
            }
        }
    }
}
