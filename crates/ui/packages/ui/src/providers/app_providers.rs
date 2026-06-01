use dioxus::prelude::*;

use super::{
    ChatRuntimeProvider, ChatStateProvider, ConversationTransitionProvider, PreferenceProvider,
    UserProvider,
};

#[component]
pub fn AppProviders(children: Element) -> Element {
    rsx! {
        ConversationTransitionProvider {
            ChatStateProvider {
                UserProvider {
                    PreferenceProvider {
                        ChatRuntimeProvider {
                            {children}
                        }
                    }
                }
            }
        }
    }
}
