use dioxus::prelude::*;
use ui::views::{ChatView, HomeView, MeView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopLayout)]
    #[route("/")]
    HomeView {},
    #[route("/chat")]
    ChatView {},
    #[route("/me")]
    MeView {},
}

const DESKTOP_CSS: Asset = asset!("/assets/desktop.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn DesktopLayout() -> Element {
    rsx! {
            nav {
                Link {
                    to: Route::HomeView {},
                    "home"
                }
                " · "
                Link {
                    to: Route::ChatView {},
                    "/chat"
                }
                " · "
                Link {
                    to: Route::MeView {},
                    "/me"
                }
            }
            Outlet::<Route> {}
        document::Link { rel: "stylesheet", href: DESKTOP_CSS }
    }
}
