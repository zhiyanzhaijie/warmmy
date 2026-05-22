use dioxus::prelude::*;
use ui::views::{ChatDetailView, HomeView, MeView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebLayout)]
    #[route("/")]
    HomeView {},
    #[route("/:session_id")]
    ChatDetailView { session_id: String },
    #[route("/me")]
    MeView {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const WEB_CSS: Asset = asset!("/assets/web.css");

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
                    to: Route::MeView {},
                    "/me"
                }
            }
            Outlet::<Route> {}
        document::Link { rel: "stylesheet", href: WEB_CSS }
    }
}
