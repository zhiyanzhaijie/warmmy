use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowLeft, Bot, HeartHandshake, Pencil, Save, Target, UserRound, Utensils,
};

use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};

use super::common::{BlockMessage, LabeledInput, LabeledTextarea};
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
                div { class: "relative flex min-h-0 flex-1 flex-col justify-between gap-6",
                    BlockMessage { message: message() }
                    WarmmyProfileArt {}
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

                    div { class: "relative z-10 grid min-h-0 grid-cols-3 gap-2 rounded-[1.75rem] border border-border bg-background/70 p-2 md:gap-3 md:p-3",
                        ProfileEntry {
                            icon: rsx! { Utensils { size: 34 } },
                            value: preference_count.to_string(),
                            label: "饮食偏好".to_string(),
                            description: "口味与避忌".to_string(),
                            accent_class: "text-primary".to_string(),
                            onclick: move |_| {
                                nav.push("/me/preferences");
                            },
                        }
                        ProfileEntry {
                            icon: rsx! { Target { size: 34 } },
                            value: active_count.to_string(),
                            label: "健康期望".to_string(),
                            description: "当前执行中".to_string(),
                            accent_class: "text-foreground".to_string(),
                            onclick: move |_| {
                                nav.push("/me/expectations");
                            },
                        }
                        ProfileEntry {
                            icon: rsx! { HeartHandshake { size: 34 } },
                            value: companion_count.to_string(),
                            label: "一起吃饭".to_string(),
                            description: "相关的人".to_string(),
                            accent_class: "text-accent-foreground".to_string(),
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
                div { class: "flex h-full w-full items-center justify-center rounded-[1.75rem] border border-border bg-foreground font-doodle text-5xl font-semibold text-background shadow-sm",
                    "{avatar_initial}"
                }
                Button {
                    class: "absolute -bottom-2 -right-2 rounded-full border border-border bg-card p-0 text-foreground shadow-sm hover:bg-muted",
                    disabled: loading,
                    onclick: move |_| {
                        nav.push("/me/profile");
                    },
                    Pencil { size: 15 }
                }
            }
            div { class: "flex flex-wrap items-center gap-2",
                div { class: "inline-flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground",
                    UserRound { size: 14 }
                    "Owner · #{user_id}"
                }
            }
            Button {
                class: "w-fit rounded-full bg-foreground px-3 py-1 text-xs text-background shadow-sm hover:opacity-90",
                onclick: move |_| {
                    nav.push("/warmmy");
                },
                Bot { size: 14 }
                "Warmmy"
            }
            div { class: "flex flex-wrap gap-2 text-xs text-muted-foreground",
                if !gender.trim().is_empty() {
                    span { class: "rounded-full border border-border bg-background px-3 py-1", "{gender}" }
                }
                if !age.trim().is_empty() {
                    span { class: "rounded-full border border-border bg-background px-3 py-1", "{age} 岁" }
                }
            }
            h2 { class: "font-doodle text-4xl font-semibold leading-tight text-foreground md:text-5xl", "{display_name}" }
            p {
                class: "max-w-3xl whitespace-normal break-words text-sm leading-relaxed text-muted-foreground md:text-base",
                "{intro}"
            }
        }
    }
}

#[component]
fn WarmmyProfileArt() -> Element {
    rsx! {
        div { class: "pointer-events-none absolute right-[-0.5rem] top-[-0.5rem] z-0 h-[11.7rem] w-[14.3rem] opacity-95 sm:h-[14.3rem] sm:w-[18.2rem] md:h-[18.2rem] md:w-[23.4rem]",
            svg {
                class: "absolute inset-0 h-full w-full",
                view_box: "0 0 360 430",
                role: "img",
                title { "Warmmy" }
                path {
                    d: "M61 202 C49 176 45 144 53 115 C57 97 65 82 77 68 C82 39 97 17 119 11 C145 4 174 22 196 47 C220 18 251 5 276 15 C300 25 314 52 319 84 C333 101 342 124 345 151 C349 189 334 222 304 244",
                    fill: "none",
                    stroke: "#d58f3f",
                    stroke_width: "8",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                path {
                    d: "M92 112 C105 74 119 52 137 44 C154 53 176 72 190 93 C154 94 121 101 92 112 Z",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "6",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                path {
                    d: "M229 94 C247 69 269 51 286 45 C302 56 315 82 322 119 C294 105 264 98 229 94 Z",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "6",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                path {
                    d: "M63 198 C56 226 63 259 83 272 C97 282 112 278 124 267 M296 265 C309 278 326 282 340 270 C356 255 359 220 343 196",
                    fill: "none",
                    stroke: "#d58f3f",
                    stroke_width: "7",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                circle { cx: "133", cy: "169", r: "37", fill: "none", stroke: "currentColor", stroke_width: "7" }
                circle { cx: "133", cy: "169", r: "27", fill: "none", stroke: "currentColor", stroke_width: "5" }
                circle { cx: "239", cy: "169", r: "37", fill: "none", stroke: "currentColor", stroke_width: "7" }
                circle { cx: "239", cy: "169", r: "27", fill: "none", stroke: "currentColor", stroke_width: "5" }
                path {
                    d: "M102 222 C136 243 229 244 272 221",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "7",
                    stroke_linecap: "round",
                }
                path {
                    d: "M73 253 C63 309 69 377 88 397 C119 419 266 419 304 398 C323 379 327 306 314 251",
                    fill: "none",
                    stroke: "#d58f3f",
                    stroke_width: "7",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                path {
                    d: "M94 276 C129 263 163 256 204 256 C240 256 276 264 306 276 M84 291 C119 302 247 304 317 285",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "5",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                path {
                    d: "M109 292 C126 277 143 276 158 291 C174 274 192 275 207 291 C224 276 244 277 260 292 C276 278 293 278 308 292",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "4",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
                path {
                    d: "M112 318 C124 305 139 305 151 318 M157 319 C170 305 187 305 199 319 M205 319 C219 306 235 306 248 319 M254 318 C267 307 283 307 296 318",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    opacity: "0.85",
                }
                path {
                    d: "M117 346 C130 333 147 333 160 346 M166 347 C180 333 197 333 209 347 M216 347 C230 334 246 334 258 347 M263 344 C275 335 290 335 303 344",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    opacity: "0.8",
                }
                path {
                    d: "M126 375 C139 361 156 361 168 375 M176 377 C190 362 207 362 220 377 M228 376 C242 363 258 363 271 376",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    opacity: "0.75",
                }
                circle { cx: "184", cy: "348", r: "37", fill: "none", stroke: "currentColor", stroke_width: "7" }
                path {
                    d: "M141 269 L166 325 M220 267 L211 325",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "7",
                    stroke_linecap: "round",
                }
            }
        }
    }
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
        "grid h-14 w-14 place-items-center rounded-[1.15rem] border border-border bg-background/75 transition group-hover:-translate-y-0.5 group-hover:bg-card md:h-16 md:w-16 {accent_class}"
    );

    rsx! {
        button {
            r#type: "button",
            class: "group flex min-h-[8.25rem] flex-col items-center justify-center rounded-[1.35rem] border border-border bg-card/80 px-2 py-4 text-center transition hover:-translate-y-0.5 hover:bg-card md:min-h-[10rem] md:px-4 md:py-5",
            onclick: move |event| onclick.call(event),
            div { class: "{icon_class}",
                {icon}
            }
            div { class: "mt-3 min-w-0 w-full",
                div { class: "font-doodle text-xl font-semibold leading-none text-foreground md:text-2xl", "{value}" }
                div { class: "mt-1 truncate text-[13px] font-semibold leading-tight text-foreground md:text-sm", "{label}" }
                div { class: "mt-1 truncate text-[11px] leading-tight text-muted-foreground md:text-xs",
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
