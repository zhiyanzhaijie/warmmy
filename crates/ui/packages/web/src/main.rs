use dioxus::prelude::*;
use ui::views::{ChatView, HomeView, MeView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebLayout)]
    #[route("/")]
    HomeView {},
    #[route("/chat")]
    ChatView {},
    #[route("/me")]
    MeView {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const WEB_CSS: Asset = asset!("/assets/web.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        Router::<Route> {}
    }
}

#[component]
fn WebLayout() -> Element {
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
        document::Link { rel: "stylesheet", href: WEB_CSS }
    }
}
