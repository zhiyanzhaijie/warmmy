use dioxus::prelude::*;

#[component]
pub fn HomeView() -> Element {
    rsx! {
        main {
            h1 { "home" }
            p { "Warmmy 入口页。" }
            p { "前往 /chat 或 /me。" }
        }
    }
}
