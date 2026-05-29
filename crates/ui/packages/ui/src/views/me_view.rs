use dioxus::prelude::*;

use crate::blocks::{
    CompanionsBlock, DietPreferenceEditBlock, HealthExpectationEditBlock, MeBlock, ProfileEditBlock,
};

#[component]
pub fn MeView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            MeBlock {}
        }
    }
}

#[component]
pub fn MeProfileEditView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            ProfileEditBlock {}
        }
    }
}

#[component]
pub fn MeCompanionsView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            CompanionsBlock {}
        }
    }
}

#[component]
pub fn MeDietPreferenceView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            DietPreferenceEditBlock {}
        }
    }
}

#[component]
pub fn MeHealthExpectationView() -> Element {
    rsx! {
        main {
            class: "h-full min-h-0 overflow-hidden",
            HealthExpectationEditBlock {}
        }
    }
}
