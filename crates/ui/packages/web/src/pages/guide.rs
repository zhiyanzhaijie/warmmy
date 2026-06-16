use dioxus::prelude::*;

use crate::components::SEO;

const HUMAN_WARMMY: Asset = asset!("/assets/human-warmmy.svg");
const GUIDE_01_02: Asset = asset!("/assets/guide-01-02.png");
const GUIDE_01_03: Asset = asset!("/assets/guide-01-03.png");
const GUIDE_01_04: Asset = asset!("/assets/guide-01-04.png");
const GUIDE_MODEL_ACTION: Asset = asset!("/assets/guide-model-action.png");
const GUIDE_MODEL_INTRO: Asset = asset!("/assets/guide-model-intro.png");
const GUIDE_MODEL_FILL: Asset = asset!("/assets/guide-model-fill.png");
const GUIDE_EMBEDING_ENTRANCE: Asset = asset!("/assets/guide_embeding_entrance.png");
const GUIDE_EMBEDING_FILL: Asset = asset!("/assets/guide_embeding_fill.png");
const WAMMY_ONE: Asset = asset!("/assets/wammy_one.svg");
const WAMMY_TWO: Asset = asset!("/assets/wammy_two.svg");
const WAMMY_THREE: Asset = asset!("/assets/wammy_three.svg");
const WAMMY_FOUR: Asset = asset!("/assets/wammy_four.svg");

#[derive(Clone, Copy, PartialEq)]
enum GuideStep {
    ApiKey,
    TextModel,
    VectorModel,
    VisionModel,
}

#[derive(Clone, Copy, PartialEq)]
enum ApiKeySubStep {
    EnterPage,
    CreateKey,
    Recharge,
    UseForWarmmy,
}
#[derive(Clone, Copy, PartialEq)]
enum TextModelSubStep {
    ChooseModel,
    KeyInfo,
    CompleteConfig,
}
#[derive(Clone, Copy, PartialEq)]
enum VectorModelSubStep {
    ChooseModel,
    CompleteConfig,
}
#[derive(Clone, Copy, PartialEq)]
enum VisionModelSubStep {
    ChooseModel,
    KeyInfo,
    CompleteConfig,
}

#[component]
fn SidebarStepButton(
    active: bool,
    step: &'static str,
    title: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class_name = if active {
        "flex w-full flex-col items-start justify-start rounded-xl bg-foreground/[0.06] px-3 py-3 text-left text-foreground transition"
    } else {
        "flex w-full flex-col items-start justify-start rounded-xl px-3 py-3 text-left text-muted-foreground transition hover:bg-muted/60 hover:text-foreground"
    };

    rsx! {
        button {
            class: class_name,
            onclick: move |evt| onclick.call(evt),
            p { class: "text-left text-[10px] tracking-[0.16em] text-muted-foreground", "{step}" }
            p { class: "mt-1 truncate text-left text-sm leading-snug", "{title}" }
        }
    }
}

#[component]
fn SidebarSubStepButton(
    active: bool,
    code: &'static str,
    title: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class_name = if active {
        "flex w-full flex-col items-start justify-start rounded-lg bg-foreground/[0.06] px-2.5 py-2 text-left text-foreground transition"
    } else {
        "flex w-full flex-col items-start justify-start rounded-lg px-2.5 py-2 text-left text-muted-foreground transition hover:bg-muted/60 hover:text-foreground"
    };

    rsx! {
        button {
            class: class_name,
            onclick: move |evt| onclick.call(evt),
            p { class: "text-left text-[10px] tracking-[0.14em] text-muted-foreground", "{code}" }
            p { class: "mt-1 truncate text-left text-[13px] leading-snug", "{title}" }
        }
    }
}

#[component]
fn ApiKeyStepContent(current_sub_step: ApiKeySubStep) -> Element {
    let (code, title, desc, extra, entry_url, visual) = match current_sub_step {
        ApiKeySubStep::EnterPage => (
            "01.01",
            "进入 API Key 页面",
            "我们将用豆包平台作为示例。",
            "",
            "https://console.volcengine.com/ark/region:ark+cn-beijing/model",
            "",
        ),
        ApiKeySubStep::CreateKey => (
            "01.02",
            "创建 API Key",
            "在控制台创建新的 API Key，并在 Warmmy 的钥匙库中保存对应名称与密钥。",
            "建议为不同 Provider 使用不同 key，便于按供应商做权限收敛和轮换。",
            "",
            "guide-01-02",
        ),
        ApiKeySubStep::Recharge => (
            "01.03",
            "充值",
            "确认平台账户余额充足，再回到 Warmmy 保存 Provider 与模型路由。",
            "余额不足会导致模型请求失败，建议先完成充值再进行连通性验证。",
            "",
            "guide-01-03",
        ),
        ApiKeySubStep::UseForWarmmy => (
            "01.04",
            "为屋米使用密钥",
            "将已创建并可用的密钥填入屋米配置，确认 Provider 与模型路由后完成保存。",
            "",
            "",
            "guide-01-04",
        ),
    };
    let layout_class = "flex flex-col items-start gap-8";

    rsx! {
        div { class: "{layout_class}",
            div {
                p { class: "text-xs tracking-[0.2em] text-muted-foreground", "01" }
                p { class: "mt-2 text-[11px] tracking-[0.16em] text-muted-foreground", "{code}" }
                h2 { class: "mt-3 font-serif text-3xl leading-tight text-foreground", "{title}" }
                p { class: "mt-4 text-base leading-relaxed text-muted-foreground", "{desc}" }
                if !entry_url.is_empty() {
                    a {
                        class: "mt-3 block break-all text-sm leading-relaxed text-foreground underline underline-offset-2",
                        href: "{entry_url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{entry_url}"
                    }
                }
                if !extra.is_empty() {
                    p { class: "mt-3 text-sm leading-relaxed text-muted-foreground", "{extra}" }
                }
            }
            if !visual.is_empty() {
                div { class: "w-full",
                    StepVisual { visual }
                }
            }
        }
    }
}

#[component]
pub fn GuidePage() -> Element {
    let mut current = use_signal(|| GuideStep::ApiKey);
    let mut api_key_sub_step = use_signal(|| ApiKeySubStep::EnterPage);
    let mut text_model_sub_step = use_signal(|| TextModelSubStep::ChooseModel);
    let mut vector_model_sub_step = use_signal(|| VectorModelSubStep::ChooseModel);
    let mut vision_model_sub_step = use_signal(|| VisionModelSubStep::ChooseModel);
    let wammy_visual = match current() {
        GuideStep::ApiKey => WAMMY_ONE,
        GuideStep::TextModel => WAMMY_TWO,
        GuideStep::VectorModel => WAMMY_THREE,
        GuideStep::VisionModel => WAMMY_FOUR,
    };
    let mobile_step_active =
        "shrink-0 rounded-full bg-foreground px-3 py-1.5 text-xs font-medium text-background";
    let mobile_step_idle =
        "shrink-0 rounded-full bg-muted px-3 py-1.5 text-xs font-medium text-muted-foreground";
    let mobile_sub_active =
        "shrink-0 rounded-full bg-foreground/10 px-2.5 py-1 text-xs font-medium text-foreground";
    let mobile_sub_idle =
        "shrink-0 rounded-full bg-muted/70 px-2.5 py-1 text-xs font-medium text-muted-foreground";

    rsx! {
        SEO {
            title: "配置指南",
            description: "Warmmy 配置指南，说明如何为屋米配置 API Key、文本模型、向量模型和视觉模型。",
            keywords: "Warmmy 配置指南,屋米配置,API Key,文本模型,向量模型,视觉模型,火山方舟",
        }
        div { class: "fixed inset-x-0 top-0 z-50 border-b border-border/50 bg-background/95 backdrop-blur sm:hidden",
            div { class: "mx-auto flex w-full max-w-6xl items-center gap-2 overflow-x-auto px-3 py-2 whitespace-nowrap",
                button {
                    class: if current() == GuideStep::ApiKey { mobile_step_active } else { mobile_step_idle },
                    onclick: move |_| {
                        current.set(GuideStep::ApiKey);
                        api_key_sub_step.set(ApiKeySubStep::EnterPage);
                    },
                    "01"
                }
                button {
                    class: if current() == GuideStep::TextModel { mobile_step_active } else { mobile_step_idle },
                    onclick: move |_| {
                        current.set(GuideStep::TextModel);
                        text_model_sub_step.set(TextModelSubStep::ChooseModel);
                    },
                    "02"
                }
                button {
                    class: if current() == GuideStep::VectorModel { mobile_step_active } else { mobile_step_idle },
                    onclick: move |_| {
                        current.set(GuideStep::VectorModel);
                        vector_model_sub_step.set(VectorModelSubStep::ChooseModel);
                    },
                    "03"
                }
                button {
                    class: if current() == GuideStep::VisionModel { mobile_step_active } else { mobile_step_idle },
                    onclick: move |_| {
                        current.set(GuideStep::VisionModel);
                        vision_model_sub_step.set(VisionModelSubStep::ChooseModel);
                    },
                    "04"
                }
                div { class: "mx-1 h-4 w-px shrink-0 bg-border/70" }
                if current() == GuideStep::ApiKey {
                    button {
                        class: if api_key_sub_step() == ApiKeySubStep::EnterPage { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| api_key_sub_step.set(ApiKeySubStep::EnterPage),
                        "01.01"
                    }
                    button {
                        class: if api_key_sub_step() == ApiKeySubStep::CreateKey { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| api_key_sub_step.set(ApiKeySubStep::CreateKey),
                        "01.02"
                    }
                    button {
                        class: if api_key_sub_step() == ApiKeySubStep::Recharge { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| api_key_sub_step.set(ApiKeySubStep::Recharge),
                        "01.03"
                    }
                    button {
                        class: if api_key_sub_step() == ApiKeySubStep::UseForWarmmy { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| api_key_sub_step.set(ApiKeySubStep::UseForWarmmy),
                        "01.04"
                    }
                } else if current() == GuideStep::TextModel {
                    button {
                        class: if text_model_sub_step() == TextModelSubStep::ChooseModel { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| text_model_sub_step.set(TextModelSubStep::ChooseModel),
                        "02.01"
                    }
                    button {
                        class: if text_model_sub_step() == TextModelSubStep::KeyInfo { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| text_model_sub_step.set(TextModelSubStep::KeyInfo),
                        "02.02"
                    }
                    button {
                        class: if text_model_sub_step() == TextModelSubStep::CompleteConfig { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| text_model_sub_step.set(TextModelSubStep::CompleteConfig),
                        "02.03"
                    }
                } else if current() == GuideStep::VectorModel {
                    button {
                        class: if vector_model_sub_step() == VectorModelSubStep::ChooseModel { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| vector_model_sub_step.set(VectorModelSubStep::ChooseModel),
                        "03.01"
                    }
                    button {
                        class: if vector_model_sub_step() == VectorModelSubStep::CompleteConfig { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| vector_model_sub_step.set(VectorModelSubStep::CompleteConfig),
                        "03.02"
                    }
                } else if current() == GuideStep::VisionModel {
                    button {
                        class: if vision_model_sub_step() == VisionModelSubStep::ChooseModel { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| vision_model_sub_step.set(VisionModelSubStep::ChooseModel),
                        "04.01"
                    }
                    button {
                        class: if vision_model_sub_step() == VisionModelSubStep::KeyInfo { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| vision_model_sub_step.set(VisionModelSubStep::KeyInfo),
                        "04.02"
                    }
                    button {
                        class: if vision_model_sub_step() == VisionModelSubStep::CompleteConfig { mobile_sub_active } else { mobile_sub_idle },
                        onclick: move |_| vision_model_sub_step.set(VisionModelSubStep::CompleteConfig),
                        "04.03"
                    }
                }
            }
        }
        section { class: "mx-auto flex w-full max-w-6xl flex-col gap-10 pb-12 pt-20 sm:py-20",
            div { class: "flex w-full flex-col items-start gap-6 sm:flex-row sm:items-center sm:gap-8",
                div { class: "min-w-0 max-w-3xl flex-1 text-left",
                    h1 { class: "font-serif text-4xl leading-tight text-foreground sm:text-6xl", "配置指南" }
                    p { class: "mt-6 text-lg leading-relaxed text-muted-foreground",
                        "屋米是只小猫。为屋米，我们有说话围巾，记忆背包和好奇放大镜。它们是屋米的能力道具，使得屋米可以：与人类交流，记住饮食以外的记忆，从图片中识别信息。"
                    }
                }
                div { class: "flex-none self-center sm:self-auto",
                    img {
                        src: wammy_visual,
                        class: "block h-24 w-24 object-contain sm:h-56 sm:w-56",
                        alt: "屋米状态图",
                    }
                }
            }
            div { class: "flex w-full flex-col gap-4 lg:flex-row lg:items-start lg:gap-8",
                div { class: "flex w-full items-start gap-3 sm:gap-4 lg:w-auto lg:shrink-0",
                    aside { class: "min-w-0 flex-1 overflow-hidden bg-card p-3 lg:w-56 lg:flex-none",
                    div { class: "flex flex-col gap-2",
                        SidebarStepButton {
                            active: current() == GuideStep::ApiKey,
                            step: "01",
                            title: "配置 API Key（密钥）",
                            onclick: move |_| {
                                current.set(GuideStep::ApiKey);
                            },
                        }

                        SidebarStepButton {
                            active: current() == GuideStep::TextModel,
                            step: "02",
                            title: "说话围巾（文本模型）",
                            onclick: move |_| {
                                current.set(GuideStep::TextModel);
                                text_model_sub_step.set(TextModelSubStep::ChooseModel);
                            },
                        }
                        SidebarStepButton {
                            active: current() == GuideStep::VectorModel,
                            step: "03",
                            title: "记忆背包（向量模型）",
                            onclick: move |_| {
                                current.set(GuideStep::VectorModel);
                                vector_model_sub_step.set(VectorModelSubStep::ChooseModel);
                            },
                        }
                        SidebarStepButton {
                            active: current() == GuideStep::VisionModel,
                            step: "04",
                            title: "好奇放大镜（视觉模型）",
                            onclick: move |_| {
                                current.set(GuideStep::VisionModel);
                                vision_model_sub_step.set(VisionModelSubStep::ChooseModel);
                            },
                        }
                    }
                }
                    aside { class: "min-w-0 flex-1 overflow-hidden bg-card p-3 lg:w-40 lg:flex-none",
                    if current() == GuideStep::ApiKey {
                        div { class: "flex flex-col gap-1.5",
                            SidebarSubStepButton {
                                active: api_key_sub_step() == ApiKeySubStep::EnterPage,
                                code: "01.01",
                                title: "进入 API Key 页面",
                                onclick: move |_| {
                                    current.set(GuideStep::ApiKey);
                                    api_key_sub_step.set(ApiKeySubStep::EnterPage);
                                },
                            }
                            SidebarSubStepButton {
                                active: api_key_sub_step() == ApiKeySubStep::CreateKey,
                                code: "01.02",
                                title: "创建 API Key",
                                onclick: move |_| {
                                    current.set(GuideStep::ApiKey);
                                    api_key_sub_step.set(ApiKeySubStep::CreateKey);
                                },
                            }
                            SidebarSubStepButton {
                                active: api_key_sub_step() == ApiKeySubStep::Recharge,
                                code: "01.03",
                                title: "充值",
                                onclick: move |_| {
                                    current.set(GuideStep::ApiKey);
                                    api_key_sub_step.set(ApiKeySubStep::Recharge);
                                },
                            }
                            SidebarSubStepButton {
                                active: api_key_sub_step() == ApiKeySubStep::UseForWarmmy,
                                code: "01.04",
                                title: "为屋米使用密钥",
                                onclick: move |_| {
                                    current.set(GuideStep::ApiKey);
                                    api_key_sub_step.set(ApiKeySubStep::UseForWarmmy);
                                },
                            }
                        }
                    } else if current() == GuideStep::TextModel {
                        div { class: "flex flex-col gap-1.5",
                            SidebarSubStepButton {
                                active: text_model_sub_step() == TextModelSubStep::ChooseModel,
                                code: "02.01",
                                title: "选择合适的文本模型",
                                onclick: move |_| {
                                    current.set(GuideStep::TextModel);
                                    text_model_sub_step.set(TextModelSubStep::ChooseModel);
                                },
                            }
                            SidebarSubStepButton {
                                active: text_model_sub_step() == TextModelSubStep::KeyInfo,
                                code: "02.02",
                                title: "模型关键信息",
                                onclick: move |_| {
                                    current.set(GuideStep::TextModel);
                                    text_model_sub_step.set(TextModelSubStep::KeyInfo);
                                },
                            }
                            SidebarSubStepButton {
                                active: text_model_sub_step() == TextModelSubStep::CompleteConfig,
                                code: "02.03",
                                title: "完成屋米配置",
                                onclick: move |_| {
                                    current.set(GuideStep::TextModel);
                                    text_model_sub_step.set(TextModelSubStep::CompleteConfig);
                                },
                            }
                        }
                    } else if current() == GuideStep::VectorModel {
                        div { class: "flex flex-col gap-1.5",
                            SidebarSubStepButton {
                                active: vector_model_sub_step() == VectorModelSubStep::ChooseModel,
                                code: "03.01",
                                title: "选择合适的向量模型",
                                onclick: move |_| {
                                    current.set(GuideStep::VectorModel);
                                    vector_model_sub_step.set(VectorModelSubStep::ChooseModel);
                                },
                            }
                            SidebarSubStepButton {
                                active: vector_model_sub_step() == VectorModelSubStep::CompleteConfig,
                                code: "03.02",
                                title: "完成屋米配置",
                                onclick: move |_| {
                                    current.set(GuideStep::VectorModel);
                                    vector_model_sub_step.set(VectorModelSubStep::CompleteConfig);
                                },
                            }
                        }
                    } else if current() == GuideStep::VisionModel {
                        div { class: "flex flex-col gap-1.5",
                            SidebarSubStepButton {
                                active: vision_model_sub_step() == VisionModelSubStep::ChooseModel,
                                code: "04.01",
                                title: "选择合适的视觉模型",
                                onclick: move |_| {
                                    current.set(GuideStep::VisionModel);
                                    vision_model_sub_step.set(VisionModelSubStep::ChooseModel);
                                },
                            }
                            SidebarSubStepButton {
                                active: vision_model_sub_step() == VisionModelSubStep::KeyInfo,
                                code: "04.02",
                                title: "模型关键信息",
                                onclick: move |_| {
                                    current.set(GuideStep::VisionModel);
                                    vision_model_sub_step.set(VisionModelSubStep::KeyInfo);
                                },
                            }
                            SidebarSubStepButton {
                                active: vision_model_sub_step() == VisionModelSubStep::CompleteConfig,
                                code: "04.03",
                                title: "完成屋米配置",
                                onclick: move |_| {
                                    current.set(GuideStep::VisionModel);
                                    vision_model_sub_step.set(VisionModelSubStep::CompleteConfig);
                                },
                            }
                        }
                    } else {
                        div { class: "mt-2 h-24 rounded-lg bg-muted/40" }
                    }
                }
                }

                div { class: "min-w-0 w-full flex-1 overflow-hidden p-2 sm:p-4",
                    match current() {
                        GuideStep::ApiKey => rsx! {
                            ApiKeyStepContent { current_sub_step: api_key_sub_step() }
                        },
                        GuideStep::TextModel => rsx! {
                            TextModelStepContent { current_sub_step: text_model_sub_step() }
                        },
                        GuideStep::VectorModel => rsx! {
                            VectorModelStepContent { current_sub_step: vector_model_sub_step() }
                        },
                        GuideStep::VisionModel => rsx! {
                            VisionModelStepContent { current_sub_step: vision_model_sub_step() }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn TextModelStepContent(current_sub_step: TextModelSubStep) -> Element {
    let (code, title, desc, extra, visual) = match current_sub_step {
        TextModelSubStep::ChooseModel => (
            "02.01",
            "选择合适的文本模型",
            "根据你对响应速度、成本和推理质量的要求选择文本模型。日常记录与追踪建议优先选择稳定、延迟更低的模型，复杂分析场景再切换到更强推理模型。",
            "",
            "guide-model-action",
        ),
        TextModelSubStep::KeyInfo => (
            "02.02",
            "模型关键信息",
            "确认模型名称、上下文长度、输入输出价格与可用区域，并核对是否支持你当前的调用方式。",
            "建议把模型名与供应商信息保持一一对应，避免路由时出现配置歧义。",
            "guide-model-intro",
        ),
        TextModelSubStep::CompleteConfig => (
            "02.03",
            "完成屋米配置",
            "在屋米中填写文本模型配置并保存，随后发起一次连通性测试，确认对话链路可用后再进入向量与视觉模型配置。",
            "配置完成后，屋米将获得说话围巾。",
            "guide-model-fill",
        ),
    };
    let layout_class = "flex flex-col items-start gap-8";
    rsx! {
        div { class: "{layout_class}",
            div {
                p { class: "text-xs tracking-[0.2em] text-muted-foreground", "02" }
                p { class: "mt-2 text-[11px] tracking-[0.16em] text-muted-foreground", "{code}" }
                h2 { class: "mt-3 font-serif text-3xl leading-tight text-foreground", "{title}" }
                p { class: "mt-4 text-base leading-relaxed text-muted-foreground", "{desc}" }
                if !extra.is_empty() {
                    p { class: "mt-3 text-sm leading-relaxed text-muted-foreground", "{extra}" }
                }
            }
            if !visual.is_empty() {
                div { class: "w-full",
                    StepVisual { visual }
                }
            }
        }
    }
}

#[component]
fn VectorModelStepContent(current_sub_step: VectorModelSubStep) -> Element {
    let (code, title, desc, extra, visual) = match current_sub_step {
        VectorModelSubStep::ChooseModel => (
            "03.01",
            "选择合适的向量模型",
            "进入模型广场，选择火山方舟提供的向量化模型。Warmmy 开发阶段使用的是 Doubao-embedding-vision，点击模型详情中的 API 接入获取配置所需的信息。",
            "重点确认模型 ID 与支持维度，后续需要填入屋米的向量模型配置。",
            "guide-embeding-entrance",
        ),
        VectorModelSubStep::CompleteConfig => (
            "03.02",
            "完成屋米配置",
            "在屋米中添加模型配置，Provider、API key 与 Base URL 保持和前面填写一致；Model 填入对应模型 ID。",
            "Embedding dims 按模型支持维度填写，没有特殊需求时保持默认 1024 即可。配置完成后，屋米将获得记忆背包。",
            "guide-embeding-fill",
        ),
    };
    let layout_class = "flex flex-col items-start gap-8";
    rsx! {
        div { class: "{layout_class}",
            div {
                p { class: "text-xs tracking-[0.2em] text-muted-foreground", "03" }
                p { class: "mt-2 text-[11px] tracking-[0.16em] text-muted-foreground", "{code}" }
                h2 { class: "mt-3 font-serif text-3xl leading-tight text-foreground", "{title}" }
                p { class: "mt-4 text-base leading-relaxed text-muted-foreground", "{desc}" }
                if !extra.is_empty() {
                    p { class: "mt-3 text-sm leading-relaxed text-muted-foreground", "{extra}" }
                }
            }
            if !visual.is_empty() {
                div { class: "w-full",
                    StepVisual { visual }
                }
            }
        }
    }
}

#[component]
fn VisionModelStepContent(current_sub_step: VisionModelSubStep) -> Element {
    let (code, title, desc, extra, visual) = match current_sub_step {
        VisionModelSubStep::ChooseModel => (
            "04.01",
            "选择合适的视觉模型",
            "在豆包模型中，我们的文本模型同样也支持视觉，所以完全可以一样地配置呢",
            "",
            "guide-model-action",
        ),
        VisionModelSubStep::KeyInfo => (
            "04.02",
            "模型关键信息",
            "确认支持的图片格式",
            "",
            "guide-model-intro",
        ),
        VisionModelSubStep::CompleteConfig => (
            "04.03",
            "完成屋米配置",
            "在屋米中填写视觉模型配置并保存，并将模型状态切换为Active激活。",
            "配置完成后，屋米将获得好奇放大镜。",
            "guide-model-fill",
        ),
    };
    let layout_class = "flex flex-col items-start gap-8";
    rsx! {
        div { class: "{layout_class}",
            div {
                p { class: "text-xs tracking-[0.2em] text-muted-foreground", "04" }
                p { class: "mt-2 text-[11px] tracking-[0.16em] text-muted-foreground", "{code}" }
                h2 { class: "mt-3 font-serif text-3xl leading-tight text-foreground", "{title}" }
                p { class: "mt-4 text-base leading-relaxed text-muted-foreground", "{desc}" }
                if !extra.is_empty() {
                    p { class: "mt-3 text-sm leading-relaxed text-muted-foreground", "{extra}" }
                }
            }
            if !visual.is_empty() {
                div { class: "w-full",
                    StepVisual { visual }
                }
            }
        }
    }
}

#[component]
fn StepVisual(visual: &'static str) -> Element {
    match visual {
        "guide-01-02" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_01_02,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "创建 API Key 指引配图",
                }
            }
        },
        "guide-model-action" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_MODEL_ACTION,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "文本模型操作指引配图",
                }
            }
        },
        "guide-model-intro" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_MODEL_INTRO,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "文本模型信息说明配图",
                }
            }
        },
        "guide-model-fill" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_MODEL_FILL,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "文本模型配置完成配图",
                }
            }
        },
        "guide-embeding-entrance" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_EMBEDING_ENTRANCE,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "向量模型选择指引配图",
                }
            }
        },
        "guide-embeding-fill" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_EMBEDING_FILL,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "向量模型屋米配置配图",
                }
            }
        },
        "guide-01-03" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_01_03,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "充值指引配图",
                }
            }
        },
        "guide-01-04" => rsx! {
            div { class: "w-full overflow-hidden",
                img {
                    src: GUIDE_01_04,
                    class: "h-auto w-full max-h-[28rem] object-contain",
                    alt: "为屋米使用密钥配图",
                }
            }
        },
        "key" => rsx! {
            div { class: "flex h-56 items-center justify-center rounded-2xl border border-foreground/10 bg-background",
                svg { class: "h-24 w-24 text-foreground/70", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M8 11V7a4 4 0 118 0v4m-9 0h10a1 1 0 011 1v7a1 1 0 01-1 1H7a1 1 0 01-1-1v-7a1 1 0 011-1z" }
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 15v2" }
                }
            }
        },
        "text" => rsx! {
            div { class: "relative flex h-56 items-center justify-center overflow-hidden rounded-2xl border border-foreground/10",
                img {
                    src: HUMAN_WARMMY,
                    class: "h-[170%] w-[170%] -translate-x-6 rotate-[18deg] object-contain opacity-90",
                    alt: "屋米对话能力示意图",
                }
            }
        },
        "vector" => rsx! {
            div { class: "flex h-56 items-center justify-center rounded-2xl border border-foreground/10 bg-background",
                svg { class: "h-24 w-24 text-foreground/70", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7 7a5 5 0 0110 0v1h2a1 1 0 011 1v10a2 2 0 01-2 2H6a2 2 0 01-2-2V9a1 1 0 011-1h2V7z" }
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 11h6M12 11v8" }
                }
            }
        },
        _ => rsx! {
            div { class: "flex h-56 items-center justify-center rounded-2xl border border-foreground/10 bg-background",
                svg { class: "h-24 w-24 text-foreground/70", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "1",
                    rect { x: "3", y: "4", width: "18", height: "16", rx: "2" }
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M8 14l2.5-2.5a1 1 0 011.4 0L16 15l2-2a1 1 0 011.4 0L21 14.6" }
                    circle { cx: "9", cy: "8.5", r: "1.2" }
                }
            }
        },
    }
}
