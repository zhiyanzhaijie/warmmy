use crate::Route;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Send, Sparkles, User};

#[component]
pub fn RootLayout() -> Element {
    rsx! {
        div {
            class: "flex w-full h-[100dvh] bg-background text-foreground overflow-hidden font-sans",
            SideNav {}
            div {
                class: "flex-1 h-[calc(100dvh-88px)] md:h-full min-h-0 overflow-hidden relative",
                div {
                    class: "h-full min-h-0",
                    Outlet::<Route> {}
                }
            }
            BottomNav {}
        }
    }
}

#[component]
fn BottomNav() -> Element {
    let route = use_route::<Route>();
    let is_chat = matches!(&route, Route::HomeView { .. }) || matches!(&route, Route::ChatDetailView { .. });
    let is_me = matches!(&route, Route::MeView { .. });

    rsx! {
        div {
            class: "md:hidden fixed bottom-0 w-full rounded-t-[2rem] bg-background border-t border-border z-50 overflow-hidden",
            div {
                class: "flex justify-around items-end px-4 py-3 pb-safe relative",
                Link {
                    to: Route::HomeView {},
                    class: format!(
                        "flex flex-col items-center justify-between p-2 relative group {}",
                        if is_chat { "text-foreground" } else { "text-muted-foreground" }
                    ),
                    Sparkles { size: 24 }
                }
                Link {
                    to: Route::MeView {},
                    class: format!(
                        "flex flex-col items-center justify-between p-2 relative group {}",
                        if is_me { "text-foreground" } else { "text-muted-foreground" }
                    ),
                    User { size: 24 }
                }
            }
        }
    }
}

#[component]
fn SideNav() -> Element {
    let route = use_route::<Route>();
    let is_chat = matches!(&route, Route::HomeView { .. }) || matches!(&route, Route::ChatDetailView { .. });
    let is_me = matches!(&route, Route::MeView { .. });
    rsx! {
        div {
            class: "hidden md:flex flex-col w-24 lg:w-64 h-full border-r border-border bg-sidebar pt-8 items-center lg:items-start z-50 shrink-0",
            div {
                class: "mb-12 px-0 lg:px-8 flex flex-col items-center lg:items-start gap-4",
                div {
                    class: "w-12 h-12 bg-foreground rounded-[1rem] flex items-center justify-center text-background shadow-sm",
                    Sparkles { size: 24 }
                }
                span {
                    class: "hidden lg:block font-doodle text-2xl font-bold text-sidebar-foreground tracking-wide",
                    "Warmmy"
                }
            }

            div {
                class: "flex flex-col w-full px-4 lg:px-6 gap-3",
                Link {
                    to: Route::HomeView {},
                    class: format!(
                        "flex items-center lg:justify-start justify-center gap-4 p-3.5 rounded-[1.25rem] transition-all relative overflow-hidden {}",
                        if is_chat {
                            "bg-muted text-sidebar-foreground font-bold"
                        } else {
                            "text-muted-foreground hover:bg-muted/50 hover:text-sidebar-foreground font-medium"
                        }
                    ),
                    Sparkles { size: 24, class: "mx-auto lg:mx-0 shrink-0" }
                    span { class: "hidden lg:block", "Diet Buddy" }
                }
                Link {
                    to: Route::MeView {},
                    class: format!(
                        "flex items-center lg:justify-start justify-center gap-4 p-3.5 rounded-[1.25rem] transition-all relative overflow-hidden {}",
                        if is_me {
                            "bg-muted text-sidebar-foreground font-bold"
                        } else {
                            "text-muted-foreground hover:bg-muted/50 hover:text-sidebar-foreground font-medium"
                        }
                    ),
                    User { size: 24, class: "mx-auto lg:mx-0 shrink-0" }
                    span { class: "hidden lg:block", "Settings" }
                }
            }
        }
    }
}
