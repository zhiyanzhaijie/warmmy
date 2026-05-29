use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowLeft, Bot, HeartHandshake, Pencil, Save, Target, UserRound, Utensils,
};

use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};

use super::common::{BlockMessage, LabeledInput, LabeledTextarea};
use crate::blocks::CurrentUserContext;

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
    let nav = navigator();

    rsx! {
        section { class: "flex h-full min-h-0 overflow-hidden",
            div { class: "flex min-h-0 flex-1 flex-col px-5 py-6 md:px-8 md:py-8",
                div { class: "flex min-h-0 flex-1 flex-col justify-between gap-6",
                    BlockMessage { message: message() }
                    div { class: "flex flex-col gap-5 sm:flex-row sm:items-center",
                        div { class: "relative h-24 w-24 shrink-0 md:h-28 md:w-28",
                            div { class: "flex h-full w-full items-center justify-center rounded-[1.75rem] border border-border bg-foreground font-doodle text-5xl font-semibold text-background shadow-sm",
                                "{avatar_initial}"
                            }
                            Button {
                                class: "absolute -bottom-2 -right-2 rounded-full border border-border bg-card p-0 text-foreground shadow-sm hover:bg-muted",
                                disabled: loading(),
                                onclick: move |_| {
                                    nav.push("/me/profile");
                                },
                                Pencil { size: 15 }
                            }
                        }
                        div { class: "min-w-0",
                            div { class: "mb-3 flex flex-wrap items-center gap-2",
                                div { class: "inline-flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground",
                                    UserRound { size: 14 }
                                    "Owner · #{user_id}"
                                }
                                Button {
                                    class: "rounded-full bg-foreground px-3 py-1 text-xs text-background shadow-sm hover:opacity-90",
                                    onclick: move |_| {
                                        nav.push("/warmmy");
                                    },
                                    Bot { size: 14 }
                                    "Warmmy"
                                }
                            }
                            h2 { class: "font-doodle text-4xl font-semibold leading-tight text-foreground md:text-5xl", "{display_name}" }
                            p { class: "mt-4 max-w-2xl text-sm leading-relaxed text-muted-foreground md:text-base", "{intro}" }
                            div { class: "mt-4 flex flex-wrap gap-2 text-xs text-muted-foreground",
                                if !gender().trim().is_empty() {
                                    span { class: "rounded-full border border-border bg-background px-3 py-1", "{gender}" }
                                }
                                if !age().trim().is_empty() {
                                    span { class: "rounded-full border border-border bg-background px-3 py-1", "{age} 岁" }
                                }
                            }
                        }
                    }

                    div { class: "grid min-h-0 grid-cols-3 gap-2 rounded-[1.75rem] border border-border bg-background/70 p-2 md:gap-3",
                        ProfileEntry {
                            icon: rsx! { Utensils { size: 30 } },
                            value: preference_count.to_string(),
                            onclick: move |_| {
                                nav.push("/me/preferences");
                            },
                        }
                        ProfileEntry {
                            icon: rsx! { Target { size: 30 } },
                            value: active_count.to_string(),
                            onclick: move |_| {
                                nav.push("/me/expectations");
                            },
                        }
                        ProfileEntry {
                            icon: rsx! { HeartHandshake { size: 30 } },
                            value: companion_count.to_string(),
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
fn ProfileEntry(icon: Element, value: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "group flex min-h-0 flex-col items-center justify-center gap-1 rounded-[1.35rem] border border-border bg-card/80 px-2 py-2 text-center transition hover:bg-muted/50 md:gap-2 md:py-3",
            onclick: move |event| onclick.call(event),
            div { class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-full border border-border bg-background text-foreground transition group-hover:bg-foreground group-hover:text-background md:h-16 md:w-16",
                    {icon}
            }
            div { class: "font-doodle text-xl font-semibold leading-none text-muted-foreground md:text-2xl", "{value}" }
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
    let nav = navigator();

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

    rsx! {
        div { class: "flex items-center justify-between gap-3",
            Button { variant: crate::components::ui::button::ButtonVariant::Ghost, class: "rounded-full border border-border px-3", onclick: move |_| {
                nav.push("/me");
            },
                ArrowLeft { size: 16 }
                "返回"
            }
            p { class: "text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground", "Owner profile" }
        }
        Card { class: "rounded-[2rem] border border-border bg-card px-0 py-0 shadow-none",
            CardHeader { class: "gap-3 px-5 pb-0 pt-5 md:px-6 md:pt-6",
                CardTitle { class: "flex items-center gap-2 text-2xl font-semibold",
                    UserRound { size: 20 }
                    "我的资料"
                }
                p { class: "text-sm leading-relaxed text-muted-foreground", "这些资料会作为 owner 的长期身份信息，被本机 agent 读取。" }
            }
            CardContent { class: "space-y-4 px-5 pb-5 pt-5 md:px-6 md:pb-6",
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
