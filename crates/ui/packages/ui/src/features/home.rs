use dioxus::prelude::*;

use crate::impls::platform::current_platform;
use crate::{Echo, Hero};

#[component]
pub fn HomeFeature() -> Element {
    let _platform = current_platform();

    rsx! {
        Hero {}
        "Hello boy olj！"
        Echo {}
    }
}
