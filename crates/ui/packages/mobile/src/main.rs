use dioxus::prelude::*;
use ui::providers::{use_chat_context_value, AppProviders};
use ui::views::{
    ChatDetailView, HomeView, MeCompanionsView, MeDietPreferenceView, MeHealthExpectationView,
    MeProfileEditView, MeView, TravelDetailView, TravelView, WarmmyView,
};

mod layouts;
mod platform;
use layouts::RootLayout;
const MOBILE_CSS: Asset = asset!("/assets/mobile.css");
const CHAT_MARKDOWN_CSS: Asset = asset!("/assets/chat-markdown.css");
const WARMMY_LOGO: Asset = asset!("/assets/warmmy_logo.svg");

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
    platform::init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let chat = use_chat_context_value();
    use_native_splash_ready_signal();

    rsx! {
        MobileSplash {}
        document::Stylesheet { href: MOBILE_CSS }
        document::Stylesheet { href: CHAT_MARKDOWN_CSS }
        AppProviders { chat,
            Router::<Route> {}
        }
    }
}

#[component]
fn MobileSplash() -> Element {
    rsx! {
        div {
            id: "warmmy-mobile-splash",
            style: "position:fixed;inset:0;z-index:2147483647;overflow:hidden;background:#fffbf5;display:flex;align-items:center;justify-content:center;pointer-events:auto;opacity:1;transition:opacity 420ms ease;",
            div {
                id: "warmmy-mobile-splash-top",
                style: "position:absolute;inset:-2px;background:#fffbf5;clip-path:polygon(0 0, 100% 0, 0 100%);transition:transform 560ms cubic-bezier(0.42,0,0.18,1);will-change:transform;",
            }
            div {
                id: "warmmy-mobile-splash-bottom",
                style: "position:absolute;inset:-2px;background:#fffbf5;clip-path:polygon(100% 0, 100% 100%, 0 100%);transition:transform 560ms cubic-bezier(0.42,0,0.18,1);will-change:transform;",
            }
            img {
                src: WARMMY_LOGO,
                alt: "",
                style: "position:relative;z-index:1;width:118px;height:92px;object-fit:contain;opacity:1;transition:opacity 220ms ease,transform 420ms ease;will-change:opacity,transform;",
            }
        }
    }
}

fn use_native_splash_ready_signal() {
    use_effect(move || {
        document::eval(
            r#"
            (() => {
                if (window.__warmmySplashReadyScheduled) return;
                window.__warmmySplashReadyScheduled = true;

                const waitForStylesheet = (name) => new Promise((resolve) => {
                    const started = Date.now();

                    const resolveAfterPaint = () => {
                        requestAnimationFrame(() => {
                            requestAnimationFrame(resolve);
                        });
                    };

                    const probe = () => {
                        const link = Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
                            .find((element) => element.href.includes(name));

                        if (!link) {
                            if (Date.now() - started > 2400) {
                                resolveAfterPaint();
                            } else {
                                setTimeout(probe, 32);
                            }
                            return;
                        }

                        if (link.sheet) {
                            resolveAfterPaint();
                            return;
                        }

                        link.addEventListener("load", resolveAfterPaint, { once: true });
                        link.addEventListener("error", resolveAfterPaint, { once: true });
                    };

                    probe();
                });

                const dismissSplash = () => {
                    const splash = document.getElementById("warmmy-mobile-splash");
                    if (!splash || splash.dataset.closing === "true") return;
                    splash.dataset.closing = "true";

                    const topPanel = document.getElementById("warmmy-mobile-splash-top");
                    const bottomPanel = document.getElementById("warmmy-mobile-splash-bottom");
                    const logo = splash.querySelector("img");

                    if (logo) {
                        logo.style.opacity = "0";
                        logo.style.transform = "scale(0.96)";
                    }

                    if (topPanel) {
                        topPanel.style.transform = "translate3d(-42vw, -30vh, 0)";
                    }

                    if (bottomPanel) {
                        bottomPanel.style.transform = "translate3d(42vw, 30vh, 0)";
                    }

                    window.setTimeout(() => {
                        splash.style.opacity = "0";
                    }, 180);

                    window.setTimeout(() => {
                        splash.remove();
                    }, 720);
                };

                Promise.all([
                    waitForStylesheet("mobile.css"),
                    waitForStylesheet("chat-markdown.css"),
                ]).then(dismissSplash);
            })();
            "#,
        );
    });
}
