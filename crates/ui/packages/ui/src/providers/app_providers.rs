use dioxus::prelude::*;

use super::{ConversationTransitionProvider, PreferenceProvider, UserProvider};

#[component]
pub fn AppProviders(children: Element) -> Element {
    rsx! {
        ConversationTransitionProvider {
            UserProvider {
                PreferenceProvider {
                    {children}
                }
            }
        }
    }
}
