use dioxus::prelude::*;

#[component]
pub fn MeBlock() -> Element {
    rsx! {
        section {
            h1 { "我 / 设置" }
            p { "模型配置、个人画像、健康偏好等入口。" }
        }
    }
}
