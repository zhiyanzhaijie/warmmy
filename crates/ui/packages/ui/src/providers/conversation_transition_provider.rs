use dioxus::prelude::*;

use crate::blocks::ConversationTransitionContext;

#[component]
pub fn ConversationTransitionProvider(children: Element) -> Element {
    let pending = use_signal(|| None);
    use_context_provider(|| ConversationTransitionContext { pending });

    rsx! {
        {children}
    }
}
