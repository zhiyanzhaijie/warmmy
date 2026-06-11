use crate::components::sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel,
    SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuButtonSize,
    SidebarMenuItem, SidebarProvider, SidebarRail, SidebarTrigger, SidebarVariant,
};
use crate::Route;
use dioxus::prelude::*;
use dioxus_icons::lucide::{House, Map, MessageCircle, Settings, User};

#[component]
pub fn RootLayout() -> Element {
    rsx! {
        SidebarProvider {
            class: "bg-background text-foreground font-sans",
            Sidebar {
                variant: SidebarVariant::Sidebar,
                collapsible: SidebarCollapsible::Icon,
                AppSidebarHeader {}
                AppSidebarContent {}
                AppSidebarFooter {}
                SidebarRail {}
            }
            SidebarInset {
                class: "min-w-0 bg-background m-0",
                div { class: "min-h-0 flex-1 overflow-hidden",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
fn AppSidebarHeader() -> Element {
    rsx! {
        SidebarHeader {
            SidebarMenu {
                SidebarMenuItem {
                    SidebarMenuButton {
                        size: SidebarMenuButtonSize::Lg,
                        tooltip: rsx! { "Warmmy" },
                        r#as: move |attributes: Vec<Attribute>| rsx! {
                            div { ..attributes,
                                SidebarTrigger { class: "shrink-0" }
                                div { class: "grid min-w-0 flex-1 text-left leading-tight",
                                    span { class: "truncate text-sm font-semibold", "Warmmy" }
                                    span { class: "truncate text-xs text-muted-foreground", "Local first" }
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
fn AppSidebarContent() -> Element {
    let route = use_route::<Route>();

    let is_home = matches!(&route, Route::HomeView { .. });
    let is_chat = matches!(&route, Route::ChatDetailView { .. });
    let is_travel = matches!(
        &route,
        Route::TravelView { .. } | Route::TravelDetailView { .. }
    );
    let is_warmmy = matches!(&route, Route::WarmmyView { .. });

    rsx! {
        SidebarContent {
            SidebarGroup {
                SidebarGroupLabel { "Workspace" }
                SidebarMenu {
                    SidebarNavItem {
                        active: is_home,
                        label: "Home".to_string(),
                        tooltip: "Home".to_string(),
                        to: Route::HomeView {},
                        icon: rsx! { House { size: 16 } },
                    }
                    SidebarNavItem {
                        active: is_chat,
                        label: "Chat".to_string(),
                        tooltip: "Chat".to_string(),
                        to: Route::ChatDetailView { session_id: ui::today_session_id() },
                        icon: rsx! { MessageCircle { size: 16 } },
                    }
                    SidebarNavItem {
                        active: is_travel,
                        label: "Travel".to_string(),
                        tooltip: "Travel".to_string(),
                        to: Route::TravelView {},
                        icon: rsx! { Map { size: 16 } },
                    }
                }
            }

            SidebarGroup {
                SidebarGroupLabel { "System" }
                SidebarMenu {
                    SidebarNavItem {
                        active: is_warmmy,
                        label: "Warmmy".to_string(),
                        tooltip: "Warmmy".to_string(),
                        to: Route::WarmmyView {},
                        icon: rsx! { Settings { size: 16 } },
                    }
                }
            }
        }
    }
}

#[component]
fn AppSidebarFooter() -> Element {
    let route = use_route::<Route>();
    let is_me = matches!(
        &route,
        Route::MeView { .. }
            | Route::MeProfileEditView { .. }
            | Route::MeCompanionsView { .. }
            | Route::MeDietPreferenceView { .. }
            | Route::MeHealthExpectationView { .. }
    );

    rsx! {
        SidebarFooter {
            SidebarMenu {
                SidebarNavItem {
                    active: is_me,
                    label: "Me".to_string(),
                    tooltip: "Me".to_string(),
                    to: Route::MeView {},
                    icon: rsx! { User { size: 16 } },
                }
            }
        }
    }
}

#[component]
fn SidebarNavItem(
    active: bool,
    label: String,
    tooltip: String,
    to: Route,
    icon: Element,
) -> Element {
    let nav = navigator();

    rsx! {
        SidebarMenuItem {
            SidebarMenuButton {
                is_active: active,
                tooltip: rsx! { "{tooltip}" },
                r#as: move |attributes: Vec<Attribute>| {
                    let icon = icon.clone();
                    let label = label.clone();
                    let to = to.clone();
                    rsx! {
                        button {
                            r#type: "button",
                            onclick: move |_| {
                                nav.push(to.clone());
                            },
                            ..attributes,
                            {icon}
                            span { "{label}" }
                        }
                    }
                },
            }
        }
    }
}
