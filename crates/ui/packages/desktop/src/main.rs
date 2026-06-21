#[cfg(feature = "desktop")]
use dioxus::desktop::tao::dpi::LogicalSize;
#[cfg(all(feature = "desktop", target_os = "macos"))]
use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
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

    #[cfg(feature = "desktop")]
    {
        let mut window = dioxus::desktop::WindowBuilder::new()
            .with_title("Warmmy")
            .with_resizable(true)
            .with_inner_size(LogicalSize::new(1280.0, 820.0));

        #[cfg(target_os = "macos")]
        {
            window = window
                .with_title_hidden(true)
                .with_titlebar_transparent(true)
                .with_fullsize_content_view(true);
        }

        let config = dioxus::desktop::Config::new().with_window(window);

        dioxus::LaunchBuilder::desktop()
            .with_cfg(config)
            .launch(App);
        return;
    }

    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(App);
    }
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
