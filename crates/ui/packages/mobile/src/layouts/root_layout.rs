use crate::Route;
use dioxus::prelude::*;
use dioxus_icons::lucide::{House, Map, MessageCircle, Sparkles, User};
use ui::blocks::{provide_current_user_context, ConversationTransitionContext};

#[component]
pub fn RootLayout() -> Element {
    let pending = use_signal(|| None);
    use_context_provider(|| ConversationTransitionContext { pending });
    provide_current_user_context();
    let route = use_route::<Route>();
    let is_chat = matches!(&route, Route::ChatDetailView { .. });

    rsx! {
        div {
            class: "flex h-[100dvh] w-full overflow-hidden bg-background text-foreground font-sans",
            style: "--warmmy-bottom-nav-height: calc(88px + env(safe-area-inset-bottom));",
            SideNav {}
            div {
                class: format!(
                    "relative min-h-0 flex-1 overflow-hidden transition-[height] duration-300 ease-out md:h-full {}",
                    if is_chat {
                        "h-[100dvh]"
                    } else {
                        "h-[calc(100dvh-var(--warmmy-bottom-nav-height))]"
                    }
                ),
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
    let nav = navigator();
    let is_home = matches!(&route, Route::HomeView { .. });
    let is_chat = matches!(&route, Route::ChatDetailView { .. });
    let is_travel = matches!(
        &route,
        Route::TravelView { .. } | Route::TravelDetailView { .. }
    );
    let is_me = matches!(
        &route,
        Route::MeView { .. }
            | Route::MeProfileEditView { .. }
            | Route::MeCompanionsView { .. }
            | Route::MeDietPreferenceView { .. }
            | Route::MeHealthExpectationView { .. }
    );

    let open_chat = move |_| {
        nav.push(format!("/{}", ui::today_session_id()));
    };

    rsx! {
        div {
            class: format!(
                "fixed bottom-0 z-50 w-full overflow-hidden rounded-t-[2rem] bg-background transition-all duration-300 ease-out md:hidden {}",
                if is_chat {
                    "h-0 translate-y-full border-t-0 opacity-0"
                } else {
                    "h-[var(--warmmy-bottom-nav-height)] translate-y-0 border-t border-border opacity-100"
                }
            ),
            div {
                class: "relative flex h-full min-h-[88px] items-start justify-around px-4 pb-safe pt-3",
                Link {
                    to: Route::HomeView {},
                    class: format!(
                        "flex flex-col items-center justify-between p-2 relative group {}",
                        if is_home { "text-foreground" } else { "text-muted-foreground" }
                    ),
                    House { size: 24 }
                    span { class: "text-[11px] font-semibold mt-1", "Home" }
                }
                button {
                    r#type: "button",
                    onclick: open_chat,
                    class: format!(
                        "flex flex-col items-center justify-between p-2 relative group bg-transparent border-0 {}",
                        if is_chat { "text-foreground" } else { "text-muted-foreground" }
                    ),
                    MessageCircle { size: 24 }
                    span { class: "text-[11px] font-semibold mt-1", "Chat" }
                }
                Link {
                    to: Route::TravelView {},
                    class: format!(
                        "flex flex-col items-center justify-between p-2 relative group {}",
                        if is_travel { "text-foreground" } else { "text-muted-foreground" }
                    ),
                    Map { size: 24 }
                    span { class: "text-[11px] font-semibold mt-1", "Travel" }
                }
                Link {
                    to: Route::MeView {},
                    class: format!(
                        "flex flex-col items-center justify-between p-2 relative group {}",
                        if is_me { "text-foreground" } else { "text-muted-foreground" }
                    ),
                    User { size: 24 }
                    span { class: "text-[11px] font-semibold mt-1", "Me" }
                }
            }
        }
    }
}

#[component]
fn SideNav() -> Element {
    let route = use_route::<Route>();
    let nav = navigator();
    let is_home = matches!(&route, Route::HomeView { .. });
    let is_chat = matches!(&route, Route::ChatDetailView { .. });
    let is_travel = matches!(
        &route,
        Route::TravelView { .. } | Route::TravelDetailView { .. }
    );
    let is_me = matches!(
        &route,
        Route::MeView { .. }
            | Route::MeProfileEditView { .. }
            | Route::MeCompanionsView { .. }
            | Route::MeDietPreferenceView { .. }
            | Route::MeHealthExpectationView { .. }
    );

    let open_chat = move |_| {
        nav.push(format!("/{}", ui::today_session_id()));
    };

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
                        if is_home {
                            "bg-muted text-sidebar-foreground font-bold"
                        } else {
                            "text-muted-foreground hover:bg-muted/50 hover:text-sidebar-foreground font-medium"
                        }
                    ),
                    House { size: 24, class: "mx-auto lg:mx-0 shrink-0" }
                    span { class: "hidden lg:block", "Home" }
                }
                button {
                    r#type: "button",
                    onclick: open_chat,
                    class: format!(
                        "flex items-center lg:justify-start justify-center gap-4 p-3.5 rounded-[1.25rem] transition-all relative overflow-hidden border-0 bg-transparent text-left {}",
                        if is_chat {
                            "bg-muted text-sidebar-foreground font-bold"
                        } else {
                            "text-muted-foreground hover:bg-muted/50 hover:text-sidebar-foreground font-medium"
                        }
                    ),
                    MessageCircle { size: 24, class: "mx-auto lg:mx-0 shrink-0" }
                    span { class: "hidden lg:block", "Chat" }
                }
                Link {
                    to: Route::TravelView {},
                    class: format!(
                        "flex items-center lg:justify-start justify-center gap-4 p-3.5 rounded-[1.25rem] transition-all relative overflow-hidden {}",
                        if is_travel {
                            "bg-muted text-sidebar-foreground font-bold"
                        } else {
                            "text-muted-foreground hover:bg-muted/50 hover:text-sidebar-foreground font-medium"
                        }
                    ),
                    Map { size: 24, class: "mx-auto lg:mx-0 shrink-0" }
                    span { class: "hidden lg:block", "Travel" }
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
