use dioxus::prelude::*;
use ui::{
    layouts::AppLayout,
    views::{ChatView, HomeView, MeView},
};

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

const MAIN_CSS: Asset = asset!("/assets/main.css");

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
        AppLayout {
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
        }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
    }
}
