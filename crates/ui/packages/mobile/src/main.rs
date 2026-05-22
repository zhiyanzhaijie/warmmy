use dioxus::prelude::*;
use ui::views::{ChatDetailView, HomeView, MeView};

mod layouts;
use layouts::RootLayout;
const MOBILE_CSS: Asset = asset!("/assets/mobile.css");

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
}

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use dioxus::prelude::DioxusRouterExt;
        use dioxus::server::axum;

        let container = infra::setup::init_app_container()
            .await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        let app_state = std::sync::Arc::new(container);
        let router = axum::Router::new()
            .serve_dioxus_application(ServeConfig::default(), App)
            .layer(axum::Extension(app_state));
        Ok(router)
    });
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MOBILE_CSS }
        Router::<Route> {}
    }
}
