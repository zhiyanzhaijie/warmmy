use dioxus::prelude::*;

use super::{ChatRuntimeProvider, ChatStateProvider, PreferenceProvider, UserProvider};
use crate::blocks::ChatContext;

#[component]
pub fn AppProviders(chat: ChatContext, children: Element) -> Element {
    rsx! {
        ChatStateProvider { chat,
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
