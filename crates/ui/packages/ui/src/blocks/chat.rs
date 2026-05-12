use dioxus::prelude::*;

#[component]
pub fn ChatBlock() -> Element {
    rsx! {
        section {
            h1 { "屋米对话" }
            p { "进行 agentic 用餐评价与营养分析。" }
        }
    }
}
