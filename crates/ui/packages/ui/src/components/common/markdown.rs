use dioxus::prelude::*;
use dioxus_markdown::Markdown;

#[component]
pub fn MarkdownContent(src: ReadSignal<String>, #[props(default)] class: String) -> Element {
    rsx! {
        div {
            class: "warmmy-markdown {class}",
            Markdown {
                src,
                hard_line_breaks: true,
                preserve_html: false,
            }
        }
    }
}
