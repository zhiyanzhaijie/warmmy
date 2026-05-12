use dioxus::prelude::*;

use crate::blocks::MeBlock;

#[component]
pub fn MeView() -> Element {
    rsx! {
        main {
            MeBlock {}
        }
    }
}
