use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Save, UserRound};

use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};

use super::common::{BlockMessage, LabeledInput, LabeledTextarea, StatPill};

#[component]
pub fn ProfileBlock(
    user_id: String,
    preference_count: usize,
    active_count: usize,
    proposed_count: usize,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let display_name = use_signal(|| "屋主".to_string());
    let introduction = use_signal(String::new);
    let gender = use_signal(String::new);
    let age = use_signal(String::new);
    let mut message = use_signal(String::new);

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::get_user_profile(request_user_id).await {
                Ok(profile) => apply_profile(profile, display_name, introduction, gender, age),
                Err(err) => message.set(format!("加载用户失败: {err}")),
            }
            loading.set(false);
        });
    });

    let save_user_id = user_id.clone();
    let save_profile = move |_| {
        let request_user_id = save_user_id.clone();
        spawn(async move {
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
                    message.set("用户信息已保存".to_string());
                }
                Err(err) => message.set(format!("保存用户失败: {err}")),
            }
            saving.set(false);
        });
    };

    let avatar_initial = display_name()
        .chars()
        .next()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "屋".to_string());
    let intro = if introduction().trim().is_empty() {
        "这是本机唯一 owner 账号。补充生活方式或健康背景，agent 会更稳定地理解你。".to_string()
    } else {
        introduction()
    };

    rsx! {
        section { class: "relative overflow-hidden rounded-[2rem] border border-border bg-card shadow-none",
            div { class: "absolute -right-12 -top-16 h-44 w-44 rounded-full bg-[#FFAD1A]/20 blur-3xl" }
            div { class: "absolute -bottom-20 left-10 h-52 w-52 rounded-full bg-[#0f7a4d]/10 blur-3xl" }
            div { class: "relative grid grid-cols-1 gap-6 px-5 py-6 md:px-8 md:py-8 xl:grid-cols-[0.95fr_1.05fr]",
                div { class: "flex flex-col justify-between gap-6",
                    div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                        div { class: "flex h-24 w-24 shrink-0 items-center justify-center rounded-[2rem] border border-border bg-foreground font-doodle text-5xl font-semibold text-background shadow-sm md:h-28 md:w-28",
                            "{avatar_initial}"
                        }
                        div { class: "min-w-0",
                            div { class: "mb-3 inline-flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground",
                                UserRound { size: 14 }
                                "Owner · #{user_id}"
                            }
                            h2 { class: "font-doodle text-4xl font-semibold leading-tight tracking-[-0.9px] text-foreground md:text-5xl", "{display_name}" }
                            p { class: "mt-4 max-w-2xl text-sm leading-relaxed text-muted-foreground md:text-base", "{intro}" }
                        }
                    }

                    div { class: "grid grid-cols-3 gap-2 rounded-[1.5rem] border border-border bg-background/70 p-2 text-center",
                        StatPill { label: "偏好", value: preference_count.to_string() }
                        StatPill { label: "Active", value: active_count.to_string() }
                        StatPill { label: "Proposed", value: proposed_count.to_string() }
                    }
                }

                Card { class: "rounded-[1.5rem] border border-border bg-background/80 px-0 py-0 shadow-none",
                    CardHeader { class: "gap-3 px-4 pb-0 pt-4 md:px-5 md:pt-5",
                        CardTitle { class: "flex items-center gap-2 text-xl font-semibold tracking-[-0.3px]",
                            UserRound { size: 18 }
                            "我的资料"
                        }
                        p { class: "text-sm leading-relaxed text-muted-foreground", "本机只有一个 owner 账号；家人和朋友在下方作为关系人维护。" }
                    }
                    CardContent { class: "space-y-4 px-4 pb-4 pt-4 md:px-5 md:pb-5",
                        BlockMessage { message: message() }
                        div { class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                            LabeledInput { label: "Display name", icon: rsx! { UserRound { size: 16 } }, value: display_name, placeholder: "屋主" }
                            LabeledInput { label: "Gender", icon: rsx! { UserRound { size: 16 } }, value: gender, placeholder: "可选" }
                        }
                        LabeledInput { label: "Age", icon: rsx! { UserRound { size: 16 } }, value: age, placeholder: "可选，例如 32" }
                        LabeledTextarea { label: "Introduction", icon: rsx! { UserRound { size: 16 } }, value: introduction, placeholder: "例如：经常在家做饭，偏好轻食和中餐。" }
                        Button { class: "w-full rounded-xl bg-foreground text-background shadow-sm hover:opacity-90 sm:w-auto", disabled: saving() || loading(), onclick: save_profile,
                            Save { size: 16 }
                            if saving() { "保存中..." } else { "保存用户信息" }
                        }
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
