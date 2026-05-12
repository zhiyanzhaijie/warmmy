use dioxus::prelude::*;

#[component]
pub fn AppLayout(children: Element) -> Element {
    rsx! {
        div {
            class: "warmmy-app-layout",
            header {
                h1 { "Warmmy" }
            }
            {children}
        }
    }
}
