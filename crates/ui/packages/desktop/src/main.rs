use dioxus::prelude::*;
use ui::views::{ChatDetailView, HomeView, MeView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopLayout)]
    #[route("/")]
    HomeView {},
    #[route("/:session_id")]
    ChatDetailView { session_id: String },
    #[route("/me")]
    MeView {},
}

const DESKTOP_CSS: Asset = asset!("/assets/desktop.css");

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move { api::build_router(App).await });
    #[cfg(not(feature = "server"))]
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
                    to: Route::MeView {},
                    "/me"
                }
            }
            Outlet::<Route> {}
        document::Link { rel: "stylesheet", href: DESKTOP_CSS }
    }
}
