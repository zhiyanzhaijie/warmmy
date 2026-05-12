use dioxus::prelude::*;
use ui::views::{ChatView, HomeView, MeView};

mod layouts;
use layouts::RootLayout;
const MOBILE_CSS: Asset = asset!("/assets/mobile.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    HomeView {},
    #[route("/chat")]
    ChatView {},
    #[route("/me")]
    MeView {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MOBILE_CSS }
        Router::<Route> {}
    }
}
