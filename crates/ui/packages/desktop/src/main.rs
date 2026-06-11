use dioxus::prelude::*;
use ui::providers::{use_chat_context_value, AppProviders};
use ui::views::{
    ChatDetailView, HomeView, MeCompanionsView, MeDietPreferenceView, MeHealthExpectationView,
    MeProfileEditView, MeView, TravelDetailView, TravelView, WarmmyView,
};

mod components;
mod layouts;
use layouts::RootLayout;

const DESKTOP_CSS: Asset = asset!("/assets/desktop.css");
const CHAT_MARKDOWN_CSS: Asset = asset!("/assets/chat-markdown.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    HomeView {},
    #[route("/:session_id")]
    ChatDetailView { session_id: String },
    #[route("/travel")]
    TravelView {},
    #[route("/travel/:summary_id")]
    TravelDetailView { summary_id: String },
    #[route("/me")]
    MeView {},
    #[route("/me/profile")]
    MeProfileEditView {},
    #[route("/me/companions")]
    MeCompanionsView {},
    #[route("/me/preferences")]
    MeDietPreferenceView {},
    #[route("/me/expectations")]
    MeHealthExpectationView {},
    #[route("/warmmy")]
    WarmmyView {},
}

fn main() {
    let _ = dioxus::logger::init(dioxus::logger::tracing::Level::INFO);
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let chat = use_chat_context_value();

    rsx! {
        document::Stylesheet { href: DESKTOP_CSS }
        document::Stylesheet { href: CHAT_MARKDOWN_CSS }
        AppProviders { chat,
            Router::<Route> {}
        }
    }
}
