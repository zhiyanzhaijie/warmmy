use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowLeft, BrainCircuit, Database, Image, KeyRound, MessageCircle, Pencil, Plus, Route, Save,
    Server, X,
};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

use super::super::me::common::{BlockMessage, ChoiceOption, LabeledChoiceGroup, LabeledInput};

const PROVIDER_KIND_OPTIONS: &[ChoiceOption] = &[
    ChoiceOption::new("openai", "OpenAI"),
    ChoiceOption::new("deepseek", "DeepSeek"),
    ChoiceOption::new("siliconflow", "SiliconFlow"),
    ChoiceOption::new("openai_compatible", "兼容接口"),
];

#[derive(Clone, Copy, PartialEq)]
struct ModelTypeInfo {
    capability: &'static str,
    title: &'static str,
    subtitle: &'static str,
    current_label: &'static str,
    default_kind: &'static str,
    default_name: &'static str,
    default_base_url: &'static str,
    model_placeholder: &'static str,
    show_embedding_ndims: bool,
}

const MODEL_TYPES: [ModelTypeInfo; 3] = [
    ModelTypeInfo {
        capability: "chat",
        title: "文本对话模型",
        subtitle: "用于日常对话、饮食问答和工具调用。",
        current_label: "当前对话模型",
        default_kind: "deepseek",
        default_name: "DeepSeek",
        default_base_url: "https://api.deepseek.com",
        model_placeholder: "deepseek-chat / gpt-4.1-mini",
        show_embedding_ndims: false,
    },
    ModelTypeInfo {
        capability: "embedding",
        title: "向量 RAG 嵌入模型",
        subtitle: "用于长期语义记忆和相似度检索。",
        current_label: "当前嵌入模型",
        default_kind: "siliconflow",
        default_name: "SiliconFlow",
        default_base_url: "https://api.siliconflow.cn/v1",
        model_placeholder: "BAAI/bge-m3",
        show_embedding_ndims: true,
    },
    ModelTypeInfo {
        capability: "vision",
        title: "图像识别模型",
        subtitle: "用于餐食图片、标签和截图内容识别。",
        current_label: "当前视觉模型",
        default_kind: "openai",
        default_name: "OpenAI Vision",
        default_base_url: "https://api.openai.com/v1",
        model_placeholder: "gpt-4.1-mini / gpt-4o-mini",
        show_embedding_ndims: false,
    },
];

#[component]
pub fn AIModelBlock(user_id: String) -> Element {
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut config = use_signal(|| Option::<user::UserAIConfigDTO>::None);

    let mut selected_capability = use_signal(|| "chat".to_string());
    let mut detail_open = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);

    let mut provider_id = use_signal(String::new);
    let mut provider_kind = use_signal(|| "openai".to_string());
    let mut provider_name = use_signal(|| "OpenAI".to_string());
    let mut provider_base_url = use_signal(|| "https://api.openai.com/v1".to_string());
    let mut provider_api_key = use_signal(String::new);
    let mut provider_enabled = use_signal(|| true);
    let mut route_id = use_signal(String::new);
    let mut route_model = use_signal(String::new);
    let mut route_embedding_ndims = use_signal(|| "1024".to_string());
    let mut route_enabled = use_signal(|| true);

    let load_user_id = user_id.clone();
    use_effect(move || {
        let request_user_id = load_user_id.clone();
        spawn(async move {
            loading.set(true);
            message.set(String::new());
            match user::get_user_ai_config(request_user_id).await {
                Ok(next) => config.set(Some(next)),
                Err(err) => message.set(format!("加载模型配置失败: {err}")),
            }
            loading.set(false);
        });
    });

    let save_user_id = user_id.clone();
    let save_model = move |_| {
        let request_user_id = save_user_id.clone();
        let capability = selected_capability();
        spawn(async move {
            saving.set(true);
            message.set(String::new());

            let provider_input = user::SaveUserAIProviderInput {
                id: Some(provider_id()).filter(|value| !value.trim().is_empty()),
                kind: provider_kind(),
                name: provider_name(),
                base_url: provider_base_url(),
                api_key: Some(provider_api_key()).filter(|value| !value.trim().is_empty()),
                enabled: provider_enabled(),
            };
            let provider_kind_value = provider_input.kind.clone();
            let provider_name_value = provider_input.name.clone();
            let provider_base_url_value = provider_input.base_url.clone();

            match user::save_user_ai_provider(request_user_id.clone(), provider_input).await {
                Ok(after_provider) => {
                    let next_provider_id = pick_saved_provider_id(
                        &after_provider,
                        &provider_id(),
                        &provider_kind_value,
                        &provider_name_value,
                        &provider_base_url_value,
                    );

                    if next_provider_id.trim().is_empty() {
                        message.set("保存失败：未找到刚保存的供应商".to_string());
                    } else {
                        let ndims = if model_type_info(&capability).show_embedding_ndims {
                            route_embedding_ndims().trim().parse::<usize>().ok()
                        } else {
                            None
                        };
                        let route_input = user::SaveUserAIRouteInput {
                            id: Some(route_id()).filter(|value| !value.trim().is_empty()),
                            capability: capability.clone(),
                            provider_id: next_provider_id,
                            model: route_model(),
                            embedding_ndims: ndims,
                            enabled: route_enabled(),
                        };

                        match user::save_user_ai_route(request_user_id, route_input).await {
                            Ok(next) => {
                                config.set(Some(next));
                                provider_api_key.set(String::new());
                                dialog_open.set(false);
                                message.set("模型配置已保存".to_string());
                            }
                            Err(err) => {
                                config.set(Some(after_provider));
                                message.set(format!("保存模型路由失败: {err}"));
                            }
                        }
                    }
                }
                Err(err) => message.set(format!("保存供应商失败: {err}")),
            }

            saving.set(false);
        });
    };

    let cfg = config();
    let providers = cfg
        .as_ref()
        .map(|item| item.providers.clone())
        .unwrap_or_default();
    let routes = cfg
        .as_ref()
        .map(|item| item.routes.clone())
        .unwrap_or_default();
    let statuses = cfg
        .as_ref()
        .map(|item| item.statuses.clone())
        .unwrap_or_default();
    let active_type = model_type_info(&selected_capability());
    let active_routes = routes
        .iter()
        .filter(|route| route.capability == active_type.capability)
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        Card { class: "rounded-[1.75rem] border border-border bg-card shadow-none",
            CardHeader { class: "gap-3 px-5 pb-0 pt-5",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        CardTitle { class: "flex items-center gap-2 font-doodle text-2xl font-semibold tracking-[-0.5px]",
                            BrainCircuit { size: 20 }
                            "AI 模型"
                        }
                        p { class: "mt-2 text-sm leading-relaxed text-muted-foreground",
                            "按使用场景管理三类模型：文本对话、向量 RAG 嵌入和图像识别。API key 会通过现有密钥存储加密保存。"
                        }
                    }
                    div { class: "rounded-full border border-border bg-background px-3 py-1 text-xs text-muted-foreground",
                        if loading() { "加载中" } else { "Local-first" }
                    }
                }
            }
            CardContent { class: "space-y-5 px-5 pb-5 pt-5",
                BlockMessage { message: message() }

                if !detail_open() {
                    div { class: "grid grid-cols-1 gap-3 md:grid-cols-3",
                        for info in MODEL_TYPES {
                            ModelTypeCard {
                                info,
                                status: status_for(&statuses, info.capability),
                                onclick: move |capability: String| {
                                    selected_capability.set(capability);
                                    detail_open.set(true);
                                },
                            }
                        }
                    }
                } else {
                    div { class: "space-y-4",
                        div { class: "flex flex-col gap-3 rounded-[1.5rem] border border-border bg-background p-4 sm:flex-row sm:items-center sm:justify-between",
                            div { class: "flex items-start gap-3",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconSm,
                                    class: "mt-0.5 rounded-full border border-border",
                                    onclick: move |_| detail_open.set(false),
                                    ArrowLeft { size: 16 }
                                }
                                div {
                                    div { class: "flex items-center gap-2 text-lg font-semibold text-foreground",
                                        TypeIcon { capability: active_type.capability.to_string(), size: 18 }
                                        "{active_type.title}"
                                    }
                                    p { class: "mt-1 text-sm leading-relaxed text-muted-foreground", "{active_type.subtitle}" }
                                }
                            }
                            Button {
                                class: "rounded-xl bg-foreground px-4 text-background hover:opacity-90",
                                onclick: move |_| {
                                    reset_model_form(
                                        model_type_info(&selected_capability()),
                                        provider_id,
                                        provider_kind,
                                        provider_name,
                                        provider_base_url,
                                        provider_api_key,
                                        provider_enabled,
                                        route_id,
                                        route_model,
                                        route_embedding_ndims,
                                        route_enabled,
                                    );
                                    dialog_open.set(true);
                                },
                                Plus { size: 16 }
                                "添加模型"
                            }
                        }

                        div { class: "grid grid-cols-1 gap-3",
                            if active_routes.is_empty() {
                                EmptyModelListCard {
                                    title: active_type.title.to_string(),
                                    onclick: move |_| {
                                        reset_model_form(
                                            model_type_info(&selected_capability()),
                                            provider_id,
                                            provider_kind,
                                            provider_name,
                                            provider_base_url,
                                            provider_api_key,
                                            provider_enabled,
                                            route_id,
                                            route_model,
                                            route_embedding_ndims,
                                            route_enabled,
                                        );
                                        dialog_open.set(true);
                                    },
                                }
                            } else {
                                for route in active_routes {
                                    ModelRouteRow {
                                        route: route.clone(),
                                        provider: provider_for(&providers, &route.provider_id),
                                        onclick: move |picked: ModelRoutePick| {
                                            selected_capability.set(picked.route.capability.clone());
                                            route_id.set(picked.route.id.clone());
                                            route_model.set(picked.route.model.clone());
                                            route_embedding_ndims.set(
                                                picked.route.embedding_ndims.map(|value| value.to_string()).unwrap_or_else(|| "1024".to_string())
                                            );
                                            route_enabled.set(picked.route.enabled);
                                            provider_api_key.set(String::new());
                                            if let Some(provider) = picked.provider {
                                                provider_id.set(provider.id.clone());
                                                provider_kind.set(provider.kind.clone());
                                                provider_name.set(provider.name.clone());
                                                provider_base_url.set(provider.base_url.clone());
                                                provider_enabled.set(provider.enabled);
                                            } else {
                                                let info = model_type_info(&picked.route.capability);
                                                provider_id.set(picked.route.provider_id.clone());
                                                provider_kind.set(info.default_kind.to_string());
                                                provider_name.set(String::new());
                                                provider_base_url.set(info.default_base_url.to_string());
                                                provider_enabled.set(true);
                                            }
                                            dialog_open.set(true);
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        DialogRoot {
            open: dialog_open(),
            on_open_change: move |open| dialog_open.set(open),
            DialogContent { class: "max-h-[min(86dvh,760px)] w-[calc(100vw-1rem)] max-w-[760px] overflow-hidden rounded-[1.5rem] border border-border bg-card p-0 text-left shadow-2xl sm:w-[calc(100vw-2rem)] sm:rounded-[2rem]",
                div { class: "flex min-h-0 max-h-[min(86dvh,760px)] flex-col",
                    div { class: "flex shrink-0 items-start justify-between gap-4 border-b border-border px-4 py-4 md:px-6",
                        div {
                            DialogTitle {
                                if route_id().trim().is_empty() { "添加模型" } else { "编辑模型" }
                            }
                            DialogDescription {
                                "{model_type_info(&selected_capability()).title} · 保存后会成为该类型当前启用的路由。"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "rounded-full border border-border",
                            onclick: move |_| dialog_open.set(false),
                            X { size: 16 }
                        }
                    }
                    div { class: "min-h-0 flex-1 space-y-4 overflow-y-auto px-4 py-5 md:px-6",
                        div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                            LabeledChoiceGroup {
                                label: "Provider kind",
                                icon: rsx! { Server { size: 16 } },
                                value: provider_kind,
                                options: PROVIDER_KIND_OPTIONS.to_vec(),
                            }
                            LabeledInput { label: "Name", icon: rsx! { Server { size: 16 } }, value: provider_name, placeholder: "OpenAI / DeepSeek / SiliconFlow" }
                            LabeledInput { label: "Base URL", icon: rsx! { Route { size: 16 } }, value: provider_base_url, placeholder: model_type_info(&selected_capability()).default_base_url }
                            LabeledInput { label: "API key", icon: rsx! { KeyRound { size: 16 } }, value: provider_api_key, placeholder: "留空表示不更换已保存 key" }
                        }
                        div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                            LabeledInput {
                                label: "Model",
                                icon: rsx! { BrainCircuit { size: 16 } },
                                value: route_model,
                                placeholder: model_type_info(&selected_capability()).model_placeholder,
                            }
                            if model_type_info(&selected_capability()).show_embedding_ndims {
                                LabeledInput {
                                    label: "Embedding dims",
                                    icon: rsx! { Database { size: 16 } },
                                    value: route_embedding_ndims,
                                    placeholder: "1024",
                                }
                            }
                        }
                        div { class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                            ToggleLine { label: "启用供应商".to_string(), enabled: provider_enabled }
                            ToggleLine { label: "设为当前启用模型".to_string(), enabled: route_enabled }
                        }
                    }
                    div { class: "flex shrink-0 flex-col gap-2 border-t border-border px-4 py-4 sm:flex-row sm:justify-end md:px-6",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-xl border border-border px-4",
                            onclick: move |_| dialog_open.set(false),
                            "取消"
                        }
                        Button {
                            class: "rounded-xl bg-foreground px-5 text-background shadow-sm hover:opacity-90",
                            disabled: saving(),
                            onclick: save_model,
                            Save { size: 16 }
                            if saving() { "保存中..." } else { "保存模型" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ModelTypeCard(
    info: ModelTypeInfo,
    status: Option<user::UserAICapabilityStatusDTO>,
    onclick: EventHandler<String>,
) -> Element {
    let enabled = status.as_ref().map(|item| item.enabled).unwrap_or(false);
    let state = if enabled { "已启用" } else { "未启用" };
    let model = status
        .as_ref()
        .and_then(|item| item.model.clone())
        .unwrap_or_else(|| "尚未配置".to_string());
    let reason = status
        .as_ref()
        .and_then(|item| item.reason.clone())
        .unwrap_or_else(|| info.current_label.to_string());

    rsx! {
        button {
            r#type: "button",
            class: format!(
                "group min-h-44 rounded-[1.75rem] border p-4 text-left transition hover:-translate-y-0.5 hover:shadow-md {}",
                if enabled { "border-foreground bg-foreground text-background" } else { "border-border bg-background text-foreground hover:bg-muted/40" }
            ),
            onclick: move |_| onclick.call(info.capability.to_string()),
            div { class: "flex items-start justify-between gap-3",
                div { class: format!(
                    "flex h-10 w-10 items-center justify-center rounded-full {}",
                    if enabled { "bg-background/15 text-background" } else { "bg-foreground text-background" }
                ),
                    TypeIcon { capability: info.capability.to_string(), size: 18 }
                }
                span { class: "rounded-full border border-current/20 px-2 py-1 text-[11px] opacity-80", "{state}" }
            }
            div { class: "mt-5 text-xs uppercase tracking-[0.18em] opacity-70", "{info.current_label}" }
            div { class: "mt-2 font-doodle text-2xl font-semibold leading-tight tracking-[-0.6px]", "{info.title}" }
            div { class: "mt-2 line-clamp-2 text-sm leading-relaxed opacity-75", "{info.subtitle}" }
            div { class: "mt-4 truncate text-xs opacity-75", "{reason} · {model}" }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ModelRoutePick {
    route: user::UserAIRouteDTO,
    provider: Option<user::UserAIProviderDTO>,
}

#[component]
fn ModelRouteRow(
    route: user::UserAIRouteDTO,
    provider: Option<user::UserAIProviderDTO>,
    onclick: EventHandler<ModelRoutePick>,
) -> Element {
    let provider_name = provider
        .as_ref()
        .map(|item| item.name.clone())
        .unwrap_or_else(|| "供应商缺失".to_string());
    let provider_kind = provider
        .as_ref()
        .map(|item| provider_kind_label(&item.kind).to_string())
        .unwrap_or_else(|| route.provider_id.clone());
    let key_state = provider
        .as_ref()
        .map(|item| {
            if item.has_api_key {
                "key 已保存"
            } else {
                "缺少 key"
            }
        })
        .unwrap_or("供应商未找到");
    let click_route = route.clone();
    let click_provider = provider.clone();

    rsx! {
        button {
            r#type: "button",
            class: "w-full rounded-[1.5rem] border border-border bg-background p-4 text-left transition hover:bg-muted/50",
            onclick: move |_| onclick.call(ModelRoutePick { route: click_route.clone(), provider: click_provider.clone() }),
            div { class: "flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between",
                div { class: "min-w-0",
                    div { class: "flex items-center gap-2 font-semibold text-foreground",
                        TypeIcon { capability: route.capability.clone(), size: 17 }
                        span { class: "truncate", "{route.model}" }
                    }
                    div { class: "mt-2 truncate text-sm text-muted-foreground", "{provider_name} · {provider_kind}" }
                    div { class: "mt-2 text-xs text-muted-foreground", "{key_state}" }
                }
                div { class: "flex shrink-0 items-center gap-2",
                    span { class: format!(
                        "rounded-full px-2 py-1 text-[11px] {}",
                        if route.enabled { "bg-foreground text-background" } else { "bg-muted text-muted-foreground" }
                    ),
                        if route.enabled { "enabled" } else { "disabled" }
                    }
                    span { class: "inline-flex h-8 w-8 items-center justify-center rounded-full border border-border text-muted-foreground",
                        Pencil { size: 14 }
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyModelListCard(title: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "w-full rounded-[1.5rem] border border-dashed border-border bg-background/70 px-4 py-8 text-left text-sm leading-relaxed text-muted-foreground transition hover:bg-muted/50",
            onclick: move |event| onclick.call(event),
            div { class: "mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-foreground text-background",
                Plus { size: 18 }
            }
            div { class: "font-medium text-foreground", "添加第一个{title}" }
            div { class: "mt-1", "填写供应商、Base URL、API key 和模型名后即可启用该类型。" }
        }
    }
}

#[component]
fn ToggleLine(label: String, mut enabled: Signal<bool>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "flex w-full items-center justify-between rounded-xl border border-border bg-background px-4 py-3 text-left text-sm",
            onclick: move |_| enabled.set(!enabled()),
            span { "{label}" }
            span { class: format!(
                "rounded-full px-3 py-1 text-xs {}",
                if enabled() { "bg-foreground text-background" } else { "bg-muted text-muted-foreground" }
            ),
                if enabled() { "Enabled" } else { "Disabled" }
            }
        }
    }
}

#[component]
fn TypeIcon(capability: String, size: u32) -> Element {
    rsx! {
        if capability == "embedding" {
            Database { size }
        } else if capability == "vision" {
            Image { size }
        } else {
            MessageCircle { size }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn reset_model_form(
    info: ModelTypeInfo,
    mut provider_id: Signal<String>,
    mut provider_kind: Signal<String>,
    mut provider_name: Signal<String>,
    mut provider_base_url: Signal<String>,
    mut provider_api_key: Signal<String>,
    mut provider_enabled: Signal<bool>,
    mut route_id: Signal<String>,
    mut route_model: Signal<String>,
    mut route_embedding_ndims: Signal<String>,
    mut route_enabled: Signal<bool>,
) {
    provider_id.set(String::new());
    provider_kind.set(info.default_kind.to_string());
    provider_name.set(info.default_name.to_string());
    provider_base_url.set(info.default_base_url.to_string());
    provider_api_key.set(String::new());
    provider_enabled.set(true);
    route_id.set(String::new());
    route_model.set(String::new());
    route_embedding_ndims.set("1024".to_string());
    route_enabled.set(true);
}

fn model_type_info(capability: &str) -> ModelTypeInfo {
    for info in MODEL_TYPES {
        if info.capability == capability {
            return info;
        }
    }
    MODEL_TYPES[0]
}

fn status_for(
    statuses: &[user::UserAICapabilityStatusDTO],
    capability: &str,
) -> Option<user::UserAICapabilityStatusDTO> {
    statuses
        .iter()
        .find(|status| status.capability == capability)
        .cloned()
}

fn provider_for(
    providers: &[user::UserAIProviderDTO],
    provider_id: &str,
) -> Option<user::UserAIProviderDTO> {
    providers
        .iter()
        .find(|provider| provider.id == provider_id)
        .cloned()
}

fn provider_kind_label(kind: &str) -> &str {
    match kind {
        "openai" => "OpenAI",
        "deepseek" => "DeepSeek",
        "siliconflow" => "SiliconFlow",
        "openai_compatible" => "兼容接口",
        _ => kind,
    }
}

fn pick_saved_provider_id(
    config: &user::UserAIConfigDTO,
    preferred_id: &str,
    kind: &str,
    name: &str,
    base_url: &str,
) -> String {
    if !preferred_id.trim().is_empty() {
        return preferred_id.to_string();
    }

    config
        .providers
        .iter()
        .filter(|provider| {
            provider.kind == kind
                && provider.name == name.trim()
                && provider.base_url == base_url.trim()
        })
        .max_by(|left, right| left.updated_at.cmp(&right.updated_at))
        .map(|provider| provider.id.clone())
        .unwrap_or_default()
}
