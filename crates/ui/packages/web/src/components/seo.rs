use dioxus::prelude::*;

const SITE_NAME: &str = "Warmmy";
const TITLE_SUFFIX: &str = " | Warmmy";
const DEFAULT_DESCRIPTION: &str =
    "Warmmy 是一个本地优先的日常饮食伙伴，帮助你记录餐食、整理身体反馈，并用可配置的模型能力陪伴日常。";
const DEFAULT_KEYWORDS: &str = "Warmmy,屋米,饮食记录,饮食助手,本地优先,开源应用,AI 伙伴";

#[component]
pub fn SEO(
    title: &'static str,
    #[props(default = DEFAULT_DESCRIPTION)] description: &'static str,
    #[props(default = DEFAULT_KEYWORDS)] keywords: &'static str,
) -> Element {
    let final_title = if title.trim().is_empty() || title == SITE_NAME {
        SITE_NAME.to_string()
    } else {
        format!("{title}{TITLE_SUFFIX}")
    };

    rsx! {
        document::Title { "{final_title}" }
        document::Meta {
            name: "description",
            content: "{description}",
        }
        document::Meta {
            name: "keywords",
            content: "{keywords}",
        }
        document::Meta {
            name: "robots",
            content: "index, follow",
        }
        document::Meta {
            property: "og:title",
            content: "{final_title}",
        }
        document::Meta {
            property: "og:description",
            content: "{description}",
        }
        document::Meta {
            property: "og:type",
            content: "website",
        }
    }
}
