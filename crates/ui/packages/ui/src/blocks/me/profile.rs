use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowLeft, Bot, HeartHandshake, Pencil, Save, Target, UserRound, Utensils,
};

use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};

use super::common::{BlockMessage, LabeledInput, LabeledTextarea};
use crate::hooks::use_IO;
use crate::providers::CurrentUserContext;

#[component]
pub fn ProfileSummaryBlock(
    user_id: String,
    preference_count: usize,
    active_count: usize,
    companion_count: usize,
) -> Element {
    let mut loading = use_signal(|| false);
    let display_name = use_signal(|| "屋主".to_string());
    let introduction = use_signal(String::new);
    let gender = use_signal(String::new);
    let age = use_signal(String::new);
    let mut message = use_signal(String::new);

    let loaded_profile = use_IO({
        let user_id = user_id.clone();
        move || {
            let request_user_id = user_id.clone();
            async move { user::get_user_profile(request_user_id).await }
        }
    });
    use_effect(move || {
        loading.set(loaded_profile.read().is_none());
        if let Some(result) = loaded_profile.read().as_ref() {
            match result {
                Ok(profile) => {
                    message.set(String::new());
                    apply_profile(profile.clone(), display_name, introduction, gender, age);
                }
                Err(err) => message.set(format!("加载用户失败: {err}")),
            }
        }
    });

    let avatar_initial = display_name()
        .chars()
        .next()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "屋".to_string());
    let intro = if introduction().trim().is_empty() {
        "你好呀屋主，和屋米多聊聊你的饮食与生活习惯吧。".to_string()
    } else {
        introduction()
    };
    let nav = navigator();

    rsx! {
        section { class: "flex h-full min-h-0 overflow-hidden",
            div { class: "flex min-h-0 flex-1 flex-col px-5 py-6 md:px-8 md:py-8",
                div { class: "relative flex min-h-0 flex-1 flex-col justify-between gap-6",
                    BlockMessage { message: message() }
                    WarmmyProfileArt { user_id: user_id.clone() }
                    div { class: "relative z-10 mt-auto w-full pt-24 md:pt-28",
                        ProfileIdentityPanel {
                            user_id: user_id.clone(),
                            avatar_initial,
                            display_name: display_name(),
                            intro,
                            gender: gender(),
                            age: age(),
                            loading: loading(),
                        }
                    }

                    div { class: "relative z-10 grid min-h-0 grid-cols-3 gap-2 md:gap-3",
                        ProfileEntry {
                            icon: rsx! {
                                Utensils { size: 34 }
                            },
                            value: preference_count.to_string(),
                            label: "饮食偏好".to_string(),
                            description: "口味与避忌".to_string(),
                            accent_class: "text-foreground".to_string(),
                            onclick: move |_| {
                                nav.push("/me/preferences");
                            },
                        }
                        ProfileEntry {
                            icon: rsx! {
                                Target { size: 34 }
                            },
                            value: active_count.to_string(),
                            label: "健康期望".to_string(),
                            description: "当前执行中".to_string(),
                            accent_class: "text-foreground".to_string(),
                            onclick: move |_| {
                                nav.push("/me/expectations");
                            },
                        }
                        ProfileEntry {
                            icon: rsx! {
                                HeartHandshake { size: 34 }
                            },
                            value: companion_count.to_string(),
                            label: "一起吃饭".to_string(),
                            description: "相关的人".to_string(),
                            accent_class: "text-foreground/80".to_string(),
                            onclick: move |_| {
                                nav.push("/me/companions");
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileIdentityPanel(
    user_id: String,
    avatar_initial: String,
    display_name: String,
    intro: String,
    gender: String,
    age: String,
    loading: bool,
) -> Element {
    let nav = navigator();

    rsx! {
        div { class: "flex min-w-0 flex-col gap-4 overflow-hidden",
            div { class: "relative h-20 w-20 shrink-0 md:h-24 md:w-24",
                div { class: "flex h-full w-full items-center justify-center rounded-xl border border-border bg-foreground text-5xl font-medium tracking-tight text-background shadow-none",
                    "{avatar_initial}"
                }
                Button {
                    class: "absolute -bottom-2 -right-2 rounded-full border border-border bg-background p-2 text-foreground opacity-50 shadow-[inset_0_0.5px_0_rgba(255,255,255,0.2),inset_0_0_0_0.5px_rgba(0,0,0,0.2),0_1px_2px_rgba(0,0,0,0.05)] transition-all hover:opacity-80 disabled:opacity-50",
                    disabled: loading,
                    onclick: move |_| {
                        nav.push("/me/profile");
                    },
                    Pencil { size: 14 }
                }
            }
     div { class: "flex flex-wrap gap-2 text-[14px] text-muted-foreground",
                if !gender.trim().is_empty() {
                    span { class: "rounded-full border border-border bg-background px-3 py-1",
                        "{gender}"
                    }
                }
                if !age.trim().is_empty() {
                    span { class: "rounded-full border border-border bg-background px-3 py-1",
                        "{age} 岁"
                    }
                }
            }

            h2 { class: "text-[36px] font-semibold leading-[1.0] tracking-[-0.9px] text-foreground md:text-[48px] md:tracking-[-1.2px]",
                "{display_name}"
            }

            p { class: "max-w-3xl whitespace-normal break-words text-[16px] leading-[1.5] text-muted-foreground md:text-[18px] md:leading-[1.38]",
                "{intro}"
            }

            Button {
                class: "w-fit rounded-full bg-foreground px-4 py-1.5 text-[14px] text-background shadow-[inset_0_0.5px_0_rgba(255,255,255,0.2),inset_0_0_0_0.5px_rgba(0,0,0,0.2),0_1px_2px_rgba(0,0,0,0.05)] transition-all hover:opacity-80 focus:shadow-[0_4px_12px_rgba(0,0,0,0.1)]",
                onclick: move |_| {
                    nav.push("/warmmy");
                },
                Bot { size: 14 }
                "Warmmy"
            }

            div { class: "flex flex-wrap items-center gap-2",
                div { class: "inline-flex items-center gap-2 rounded-full border border-border bg-background px-3 py-1 text-[14px] font-medium uppercase tracking-widest text-muted-foreground",
                    UserRound { size: 14 }
                    "Owner · #{user_id}"
                }
            }
                   }
    }
}

#[component]
fn WarmmyProfileArt(user_id: String) -> Element {
    let loaded_ai_config = use_IO({
        let user_id = user_id.clone();
        move || {
            let request_user_id = user_id.clone();
            async move { user::get_user_ai_config(request_user_id).await }
        }
    });

    let warmmy_src = loaded_ai_config
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(warmmy_asset_from_config)
        .unwrap_or_else(|| asset!("/assets/wammy_one.svg"));

    rsx! {
        div { class: "pointer-events-none absolute right-[-5rem] top-[1rem] z-0 h-[20rem] w-[25rem] opacity-95 sm:h-[20.02rem] sm:w-[25.48rem] md:h-[25.48rem] md:w-[32.76rem] md:top-[2.5rem]",
            img {
                src: warmmy_src,
                class: "absolute inset-0 h-full w-full object-contain",
                alt: "",
            }
        }
    }
}

fn warmmy_asset_from_config(config: &user::UserAIConfigDTO) -> Asset {
    let has_text_model = capability_configured(config, "chat");
    let has_vector_model = capability_configured(config, "embedding");
    let has_vision_model = capability_configured(config, "vision");

    if !has_text_model {
        asset!("/assets/wammy_one.svg")
    } else if !has_vector_model {
        asset!("/assets/wammy_two.svg")
    } else if !has_vision_model {
        asset!("/assets/wammy_three.svg")
    } else {
        asset!("/assets/wammy_four.svg")
    }
}

fn capability_configured(config: &user::UserAIConfigDTO, capability: &str) -> bool {
    config
        .routes
        .iter()
        .filter(|route| route.capability == capability && route.enabled)
        .any(|route| {
            config
                .providers
                .iter()
                .any(|provider| provider.id == route.provider_id && provider.enabled)
        })
}

#[component]
fn ProfileEntry(
    icon: Element,
    value: String,
    label: String,
    description: String,
    accent_class: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let icon_class = format!(
        "grid h-14 w-14 place-items-center rounded-xl border border-border bg-background transition-colors md:h-16 md:w-16 {accent_class}"
    );

    rsx! {
        button {
            r#type: "button",
            class: "group flex min-h-[8.25rem] flex-col items-center justify-center rounded-xl border border-border bg-background px-2 py-4 text-center transition-all hover:border-foreground/40 hover:bg-foreground/5 focus:shadow-[0_4px_12px_rgba(0,0,0,0.1)] focus:outline-none md:min-h-[10rem] md:px-4 md:py-5",
            onclick: move |event| onclick.call(event),
            div { class: "{icon_class}", {icon} }
            div { class: "mt-3 min-w-0 w-full",
                div { class: "text-[20px] font-normal leading-[1.25] text-foreground",
                    "{value}"
                }
                div { class: "mt-1 truncate text-[16px] leading-[1.5] text-foreground",
                    "{label}"
                }
                div { class: "mt-1 truncate text-[14px] leading-[1.5] text-muted-foreground",
                    "{description}"
                }
            }
        }
    }
}

#[component]
pub fn ProfileEditBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let current_user_id = (current_user.user_id)();
    rsx! {
        div { class: "h-full min-h-0 overflow-y-auto px-4 py-5 pb-28 md:px-8 md:py-8 md:pb-12",
            div { class: "mx-auto flex w-full max-w-3xl flex-col gap-5",
                ProfileEditor { user_id: current_user_id }
            }
        }
    }
}

#[component]
fn ProfileEditor(user_id: String) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let display_name = use_signal(|| "屋主".to_string());
    let introduction = use_signal(String::new);
    let gender = use_signal(String::new);
    let age = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut hydrated = use_signal(|| false);
    let nav = navigator();

    let loaded_profile = use_IO({
        let user_id = user_id.clone();
        move || {
            let request_user_id = user_id.clone();
            async move { user::get_user_profile(request_user_id).await }
        }
    });
    use_effect(move || {
        loading.set(loaded_profile.read().is_none());
        if let Some(result) = loaded_profile.read().as_ref() {
            match result {
                Ok(profile) if !hydrated() => {
                    message.set(String::new());
                    apply_profile(profile.clone(), display_name, introduction, gender, age);
                    hydrated.set(true);
                }
                Ok(_) => {}
                Err(err) => message.set(format!("加载用户失败: {err}")),
            }
        }
    });

    let save_user_id = user_id.clone();
    let save_profile = move |_| {
        let request_user_id = save_user_id.clone();
        async move {
            saving.set(true);
            message.set(String::new());
            let parsed_age = age().trim().parse::<u8>().ok();
            let input = user::SaveUserProfileInput {
                id: request_user_id,
                display_name: display_name(),
                introduction: introduction(),
                gender: Some(gender()).filter(|value| !value.trim().is_empty()),
                age: parsed_age,
            };
            match user::save_user_profile(input).await {
                Ok(profile) => {
                    apply_profile(profile, display_name, introduction, gender, age);
                    hydrated.set(true);
                    message.set("用户信息已保存".to_string());
                }
                Err(err) => message.set(format!("保存用户失败: {err}")),
            }
            saving.set(false);
        }
    };

    rsx! {
        div { class: "flex items-center justify-between gap-3",
            Button {
                variant: crate::components::ui::button::ButtonVariant::Ghost,
                class: "rounded-full border border-border px-3 text-muted-foreground transition-colors hover:border-foreground/40 hover:bg-foreground/5 hover:text-foreground",
                onclick: move |_| {
                    nav.push("/me");
                },
                ArrowLeft { size: 16 }
                "返回"
            }
            p { class: "text-xs font-medium uppercase tracking-widest text-muted-foreground",
                "Owner profile"
            }
        }
        Card { class: "rounded-xl border border-border bg-background px-0 py-0 shadow-none",
            CardHeader { class: "gap-3 px-6 pb-2 pt-6",
                CardTitle { class: "flex items-center gap-2 text-[20px] font-normal leading-[1.25] text-foreground",
                    UserRound { size: 20 }
                    "我的资料"
                }
                p { class: "text-[16px] leading-[1.5] text-muted-foreground",
                    "和屋米打个招呼，介绍一下你自己，让之后的推荐更贴心。"
                }
            }
            CardContent { class: "space-y-6 px-6 pb-6 pt-4",
                BlockMessage { message: message() }
                div { class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                    LabeledInput {
                        label: "Display name",
                        icon: rsx! {
                            UserRound { size: 16 }
                        },
                        value: display_name,
                        placeholder: "屋主",
                    }
                    LabeledInput {
                        label: "Gender",
                        icon: rsx! {
                            UserRound { size: 16 }
                        },
                        value: gender,
                        placeholder: "可选",
                    }
                }
                LabeledInput {
                    label: "Age",
                    icon: rsx! {
                        UserRound { size: 16 }
                    },
                    value: age,
                    placeholder: "可选，例如 32",
                }
                LabeledTextarea {
                    label: "Introduction",
                    icon: rsx! {
                        UserRound { size: 16 }
                    },
                    value: introduction,
                    placeholder: "例如：经常在家做饭，偏好轻食和中餐。",
                }
                Button {
                    class: "w-full rounded-[6px] bg-foreground px-5 py-2 text-background shadow-[inset_0_0.5px_0_rgba(255,255,255,0.2),inset_0_0_0_0.5px_rgba(0,0,0,0.2),0_1px_2px_rgba(0,0,0,0.05)] transition-all hover:opacity-80 focus:shadow-[0_4px_12px_rgba(0,0,0,0.1)] sm:w-auto",
                    disabled: saving() || loading(),
                    onclick: save_profile,
                    Save { size: 16 }
                    if saving() {
                        "保存中..."
                    } else {
                        "保存用户信息"
                    }
                }
            }
        }
    }
}

fn apply_profile(
    profile: user::UserProfileDTO,
    mut display_name: Signal<String>,
    mut introduction: Signal<String>,
    mut gender: Signal<String>,
    mut age: Signal<String>,
) {
    display_name.set(profile.display_name);
    introduction.set(profile.introduction);
    gender.set(profile.gender.unwrap_or_default());
    age.set(
        profile
            .age
            .map(|value| value.to_string())
            .unwrap_or_default(),
    );
}
