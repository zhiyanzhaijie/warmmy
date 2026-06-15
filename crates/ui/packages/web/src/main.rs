use dioxus::prelude::*;

mod components;
mod pages;

use components::SiteLayout;
use pages::{AboutPage, DownloadPage, GuidePage, HomePage};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(SiteLayout)]
        #[route("/")]
        HomePage {},
        #[route("/guide")]
        GuidePage {},
        #[route("/about")]
        AboutPage {},
        #[route("/download")]
        DownloadPage {},
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
        document::Link { rel: "stylesheet", href: WEB_CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "true" }
        document::Link { 
            rel: "stylesheet", 
            href: "https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,500;0,600;1,400;1,500&family=Outfit:wght@300;400;500&display=swap" 
        }
        Router::<Route> {}
    }
}

pub(crate) fn github_releases_url() -> &'static str {
    "https://github.com/zhiyanzhaijie/warmmy/releases/latest"
}
