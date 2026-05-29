use dioxus::prelude::*;
use ui::views::{ChatDetailView, HomeView, MeView, WarmmyView};

mod layouts;
mod platform;
use layouts::RootLayout;
const MOBILE_CSS: Asset = asset!("/assets/mobile.css");
const CHAT_MARKDOWN_CSS: Asset = asset!("/assets/chat-markdown.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    HomeView {},
    #[route("/:session_id")]
    ChatDetailView { session_id: String },
    #[route("/me")]
    MeView {},
    #[route("/warmmy")]
    WarmmyView {},
}

fn main() {
    platform::init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MOBILE_CSS }
        document::Stylesheet { href: CHAT_MARKDOWN_CSS }
        Router::<Route> {}
    }
}
