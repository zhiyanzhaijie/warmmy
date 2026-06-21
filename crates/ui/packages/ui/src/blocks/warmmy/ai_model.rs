use api::user;
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowLeft, BrainCircuit, Database, Image, KeyRound, MessageCircle, Pencil, Plus, Route, Save,
    Server, Trash2, X,
};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::{DialogContent, DialogRoot, DialogTitle};
use crate::components::ui::input::Input;
use crate::components::ui::popover::{PopoverContent, PopoverRoot, PopoverTrigger};
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::switch::Switch;
use crate::hooks::use_IO;

use super::super::me::common::{BlockMessage, ChoiceOption, LabeledInput};

const PROVIDER_KIND_OPTIONS: &[ChoiceOption] = &[
    ChoiceOption::new("openai", "OpenAI"),
    ChoiceOption::new("deepseek", "DeepSeek"),
    ChoiceOption::new("siliconflow", "SiliconFlow"),
    ChoiceOption::new("dashscope", "DashScope"),
    ChoiceOption::new("doubao", "Doubao"),
    ChoiceOption::new("openai_compatible", "兼容接口"),
];
const FIXED_EMBEDDING_NDIMS: usize = 2048;

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
}

const MODEL_TYPES: [ModelTypeInfo; 3] = [
    ModelTypeInfo {
        capability: "chat",
        title: "文本对话模型",
        subtitle: "说话围巾",
        current_label: "当前对话模型",
        default_kind: "deepseek",
        default_name: "DeepSeek",
        default_base_url: "https://api.deepseek.com",
        model_placeholder: "deepseek-chat / qwen3.7-plus / gpt-4.1-mini",
    },
    ModelTypeInfo {
        capability: "embedding",
        title: "向量 RAG 嵌入模型",
        subtitle: "记忆背包",
        current_label: "当前嵌入模型",
        default_kind: "siliconflow",
        default_name: "SiliconFlow",
        default_base_url: "https://api.siliconflow.cn/v1",
        model_placeholder: "BAAI/bge-m3",
    },
    ModelTypeInfo {
        capability: "vision",
        title: "图像识别模型",
        subtitle: "好奇放大镜",
        current_label: "当前视觉模型",
        default_kind: "openai",
        default_name: "OpenAI Vision",
        default_base_url: "https://api.openai.com/v1",
        model_placeholder: "qwen3.7-plus / gpt-4.1-mini / gpt-4o-mini",
    },
];

#[derive(Clone, PartialEq)]
struct ModelEditorDraft {
    capability: String,
    provider_id: String,
    provider_kind: String,
    provider_name: String,
    provider_base_url: String,
    api_key_ref: String,
    provider_enabled: bool,
    route_id: String,
    route_model: String,
    route_enabled: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum ModelEditorMode {
    Add,
    Edit,
}

impl ModelEditorMode {
    fn is_edit(self) -> bool {
        matches!(self, Self::Edit)
    }

    fn label(self) -> &'static str {
        if self.is_edit() {
            "edit"
        } else {
            "add"
        }
    }
}

#[component]
pub fn AIModelBlock(user_id: String) -> Element {
    let mut loading = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut config = use_signal(|| Option::<user::UserAIConfigDTO>::None);

    let mut selected_capability = use_signal(|| "chat".to_string());
    let mut detail_open = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut key_library_open = use_signal(|| false);
    let mut hydrated = use_signal(|| false);
    let mut editor_draft = use_signal(|| new_editor_draft(model_type_info("chat")));
    let mut editor_mode = use_signal(|| ModelEditorMode::Add);
    let mut editor_session = use_signal(|| 0u64);

    let loaded_config = use_IO({
        let user_id = user_id.clone();
        move || {
            let request_user_id = user_id.clone();
            async move { user::get_user_ai_config(request_user_id).await }
        }
    });
    use_effect(move || {
        loading.set(loaded_config.read().is_none());
        if let Some(result) = loaded_config.read().as_ref() {
            match result {
                Ok(next) if !hydrated() => {
                    message.set(String::new());
                    config.set(Some(next.clone()));
                    hydrated.set(true);
                }
                Ok(_) => {}
                Err(err) => message.set(format!("加载模型配置失败: {err}")),
            }
        }
    });

    let cfg = config();
    let providers = cfg
        .as_ref()
        .map(|item| item.providers.clone())
        .unwrap_or_default();
    let api_keys = cfg
        .as_ref()
        .map(|item| item.api_keys.clone())
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
    let editor_draft_value = editor_draft();
    let current_editor_mode = editor_mode();
    let editor_dialog_key = format!(
        "{}:{}:{}:{}:{}",
        current_editor_mode.label(),
        editor_session(),
        editor_draft_value.provider_id,
        editor_draft_value.route_id,
        editor_draft_value.route_model
    );

    rsx! {
        Card { class: "rounded-2xl border border-border bg-card shadow-none",
            CardHeader { class: "gap-3 px-6 pb-2 pt-6",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        CardTitle { class: "flex items-center gap-2 text-xl font-medium tracking-tight text-foreground",
                            BrainCircuit { size: 20 }
                            "AI 模型"
                        }
                    }
                    div {
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-full border border-border bg-card p-2 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                            onclick: move |_| key_library_open.set(true),
                            KeyRound { size: 16 }
                        }
                    }
                }
            }
            CardContent { class: "space-y-6 px-6 pb-6 pt-4",
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
                    div { class: "space-y-6",
                        div { class: "flex flex-col gap-3 rounded-2xl border border-border bg-card p-4 sm:flex-row sm:items-center sm:justify-between",
                            div { class: "flex items-start gap-3",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconSm,
                                    class: "mt-0.5 rounded-full border border-border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                                    onclick: move |_| detail_open.set(false),
                                    ArrowLeft { size: 16 }
                                }
                                div {
                                    div { class: "flex items-center gap-2 text-lg font-medium tracking-tight text-foreground",
                                        TypeIcon { capability: active_type.capability.to_string(), size: 18 }
                                        "{active_type.title}"
                                    }
                                    p { class: "mt-1 text-sm leading-relaxed text-muted-foreground", "{active_type.subtitle}" }
                                }
                            }
                            Button {
                                class: "rounded-md bg-foreground px-4 py-2 text-background shadow-xs transition-all hover:opacity-90",
                                onclick: move |_| {
                                    editor_draft
                                        .set(new_editor_draft(model_type_info(&selected_capability())));
                                    editor_mode.set(ModelEditorMode::Add);
                                    editor_session.set(editor_session().saturating_add(1));
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
                                        editor_draft.set(new_editor_draft(model_type_info(
                                            &selected_capability(),
                                        )));
                                        editor_mode.set(ModelEditorMode::Add);
                                        editor_session.set(editor_session().saturating_add(1));
                                        dialog_open.set(true);
                                    },
                                }
                            } else {
                                for route in active_routes {
                                    ModelRouteRow {
                                        key: "{route.id}",
                                        route: route.clone(),
                                        provider: provider_for(&providers, &route.provider_id),
                                        onedit: move |picked: ModelRoutePick| {
                                            selected_capability.set(picked.route.capability.clone());
                                            editor_draft.set(editor_draft_from_pick(&picked));
                                            editor_mode.set(ModelEditorMode::Edit);
                                            editor_session.set(editor_session().saturating_add(1));
                                            dialog_open.set(true);
                                        },
                                        ondelete: {
                                            let request_user_id = user_id.clone();
                                            move |route_id: String| {
                                                let request_user_id = request_user_id.clone();
                                                async move {
                                                    message.set(String::new());
                                                    match user::delete_user_ai_route(request_user_id, route_id).await {
                                                        Ok(next) => {
                                                            config.set(Some(next));
                                                            hydrated.set(true);
                                                            message.set("模型配置已删除".to_string());
                                                        }
                                                        Err(err) => message.set(format!("删除模型配置失败: {err}")),
                                                    }
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        ModelEditorDialog {
            key: "{editor_dialog_key}",
            user_id: user_id.clone(),
            open: dialog_open(),
            session: editor_session(),
            mode: current_editor_mode,
            draft: editor_draft,
            api_keys: api_keys.clone(),
            on_open_change: move |open| dialog_open.set(open),
            on_saved: move |next: user::UserAIConfigDTO| {
                config.set(Some(next));
                hydrated.set(true);
            },
            on_message: move |next: String| message.set(next),
        }

        APIKeyLibraryDialog {
            user_id: user_id.clone(),
            open: key_library_open(),
            api_keys: api_keys.clone(),
            providers: providers.clone(),
            on_open_change: move |open| key_library_open.set(open),
            on_saved: move |next: user::UserAIConfigDTO| {
                config.set(Some(next));
                hydrated.set(true);
            },
        }
    }
}

#[component]
fn APIKeyLibraryDialog(
    user_id: String,
    open: bool,
    api_keys: Vec<user::UserAIKeyDTO>,
    providers: Vec<user::UserAIProviderDTO>,
    on_open_change: EventHandler<bool>,
    on_saved: EventHandler<user::UserAIConfigDTO>,
) -> Element {
    let mut saving = use_signal(|| false);
    let mut create_mode = use_signal(|| false);
    let mut editing_id = use_signal(String::new);
    let mut key_name = use_signal(String::new);
    let mut secret_value = use_signal(String::new);
    let mut local_message = use_signal(String::new);
    use_effect({
        let keys = api_keys.clone();
        move || {
            if create_mode() {
                return;
            }
            let selected_id = editing_id();
            if selected_id.trim().is_empty() {
                if let Some(first) = keys.first() {
                    editing_id.set(first.id.clone());
                    key_name.set(first.name.clone());
                }
                return;
            }
            if let Some(item) = keys.iter().find(|item| item.id == selected_id) {
                if key_name().trim().is_empty() {
                    key_name.set(item.name.clone());
                }
            }
        }
    });
    let default_key = api_keys.first().cloned();
    let selected_key = if create_mode() {
        None
    } else {
        let selected_id = editing_id();
        api_keys
            .iter()
            .find(|item| item.id == selected_id)
            .cloned()
            .or(default_key)
    };

    rsx! {
        DialogRoot {
            open,
            on_open_change: move |next| on_open_change.call(next),
            DialogContent { class: "w-[calc(100vw-1rem)] max-w-[560px] rounded-2xl border border-border bg-card p-0 text-left shadow-lg sm:w-[calc(100vw-2rem)]",
                div { class: "space-y-6 p-6",
                    div { class: "flex items-center justify-between gap-4",
                        DialogTitle { class: "text-xl font-medium tracking-tight", "钥匙库" }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "rounded-full border border-border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                            onclick: move |_| on_open_change.call(false),
                            X { size: 16 }
                        }
                    }
                    BlockMessage { message: local_message() }
                    div { class: "grid grid-cols-6 gap-2",
                        for item in api_keys.clone() {
                            button {
                                key: "{item.id}",
                                r#type: "button",
                                class: format!(
                                    "flex h-10 w-10 items-center justify-center rounded-full border transition-all {}",
                                    if !create_mode() && selected_key.as_ref().map(|key| key.id.as_str()) == Some(item.id.as_str()) {
                                        "border-foreground bg-foreground text-background shadow-xs"
                                    } else {
                                        "border-border bg-card text-muted-foreground hover:border-foreground/30 hover:text-foreground"
                                    }
                                ),
                                onclick: {
                                    let item = item.clone();
                                    move |_| {
                                        create_mode.set(false);
                                        editing_id.set(item.id.clone());
                                        key_name.set(item.name.clone());
                                    }
                                },
                                KeyRound { size: 16 }
                            }
                        }
                        button {
                            r#type: "button",
                            class: format!(
                                "flex h-10 w-10 items-center justify-center rounded-full border transition-all {}",
                                if create_mode() {
                                    "border-foreground bg-foreground text-background shadow-xs"
                                } else {
                                    "border-border border-dashed bg-card text-muted-foreground hover:border-solid hover:border-foreground/30 hover:text-foreground"
                                }
                            ),
                            onclick: move |_| {
                                create_mode.set(true);
                                key_name.set(String::new());
                                secret_value.set(String::new());
                            },
                            Plus { size: 16 }
                        }
                    }
                    div { class: "space-y-5 rounded-2xl border border-border bg-card p-5",
                        if create_mode() {
                            LabeledInput {
                                label: "Name",
                                icon: rsx! { KeyRound { size: 16 } },
                                value: key_name,
                                placeholder: "DeepSeek 主账号 / OpenAI 备用账号",
                            }
                            LabeledInput {
                                label: "Key",
                                icon: rsx! { KeyRound { size: 16 } },
                                value: secret_value,
                                placeholder: "sk-...",
                            }
                            div { class: "flex justify-end",
                                Button {
                                    class: "rounded-md bg-foreground px-5 py-2 text-background shadow-xs transition-all hover:opacity-90",
                                    disabled: saving(),
                                    onclick: {
                                        let base_user_id = user_id.clone();
                                        move |_| {
                                            let request_user_id = base_user_id.clone();
                                            let current_name = key_name();
                                            let current_secret = secret_value();
                                            async move {
                                                saving.set(true);
                                                local_message.set(String::new());
                                                match user::save_user_ai_key(request_user_id.clone(), user::SaveUserAIKeyInput {
                                                    id: None,
                                                    name: current_name.clone(),
                                                    api_key: Some(current_secret).filter(|value| !value.trim().is_empty()),
                                                }).await {
                                                    Ok(next) => {
                                                        on_saved.call(next.clone());
                                                        create_mode.set(false);
                                                        secret_value.set(String::new());
                                                        let latest_id = next
                                                            .api_keys
                                                            .iter()
                                                            .filter(|item| item.name == current_name.trim())
                                                            .max_by(|left, right| left.updated_at.cmp(&right.updated_at))
                                                            .map(|item| item.id.clone())
                                                            .unwrap_or_default();
                                                        editing_id.set(latest_id);
                                                        key_name.set(current_name);
                                                        local_message.set("API key 已保存".to_string());
                                                    }
                                                    Err(err) => local_message.set(format!("保存 API key 失败: {err}")),
                                                }
                                                saving.set(false);
                                            }
                                        }
                                    },
                                    Save { size: 16 }
                                    if saving() { "保存中..." } else { "保存" }
                                }
                            }
                        }
                        else if let Some(item) = selected_key {
                            LabeledInput {
                                label: "Name",
                                icon: rsx! { KeyRound { size: 16 } },
                                value: key_name,
                                placeholder: item.name.clone(),
                            }
                            label { class: "flex flex-col gap-2",
                                span { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                    KeyRound { size: 16 }
                                    "Key"
                                }
                                Input {
                                    class: "rounded-md border border-border bg-muted/30 px-3 py-2.5 text-sm shadow-none",
                                    value: "••••••••••••••••••••".to_string(),
                                    readonly: true,
                                }
                            }
                            div { class: "flex justify-end gap-2",
                                Button {
                                    class: "rounded-md bg-foreground px-5 py-2 text-background shadow-xs transition-all hover:opacity-90",
                                    disabled: saving(),
                                    onclick: {
                                        let base_user_id = user_id.clone();
                                        let selected_key_id = item.id.clone();
                                        move |_| {
                                            let request_user_id = base_user_id.clone();
                                            let api_key_id = selected_key_id.clone();
                                            let current_name = key_name();
                                            async move {
                                                saving.set(true);
                                                local_message.set(String::new());
                                                match user::save_user_ai_key(request_user_id.clone(), user::SaveUserAIKeyInput {
                                                    id: Some(api_key_id),
                                                    name: current_name,
                                                    api_key: None,
                                                }).await {
                                                    Ok(next) => {
                                                        on_saved.call(next);
                                                        local_message.set("API key 名称已更新".to_string());
                                                    }
                                                    Err(err) => local_message.set(format!("更新 API key 名称失败: {err}")),
                                                }
                                                saving.set(false);
                                            }
                                        }
                                    },
                                    Save { size: 14 }
                                    "保存"
                                }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    class: "rounded-md border border-border px-5 py-2 text-destructive transition-colors hover:border-destructive/30 hover:bg-destructive/5",
                                    disabled: saving(),
                                    onclick: {
                                        let base_user_id = user_id.clone();
                                        let selected_key_id = item.id.clone();
                                        let selected_secret_ref = item.secret_ref.clone();
                                        let linked_count = providers
                                            .iter()
                                            .filter(|provider| {
                                                provider
                                                    .api_key_ref
                                                    .as_deref()
                                                    == Some(selected_secret_ref.as_str())
                                            })
                                            .count();
                                        move |_| {
                                            let request_user_id = base_user_id.clone();
                                            let api_key_id = selected_key_id.clone();
                                            async move {
                                                if linked_count > 0 {
                                                    local_message.set("该 key 正被 provider 引用，请先解除引用".to_string());
                                                    return;
                                                }
                                                saving.set(true);
                                                local_message.set(String::new());
                                                match user::delete_user_ai_key(request_user_id.clone(), api_key_id).await {
                                                    Ok(next) => {
                                                        on_saved.call(next);
                                                        editing_id.set(String::new());
                                                        local_message.set("API key 已删除".to_string());
                                                    }
                                                    Err(err) => local_message.set(format!("删除 API key 失败: {err}")),
                                                }
                                                saving.set(false);
                                            }
                                        }
                                    },
                                    X { size: 14 }
                                    "删除"
                                }
                            }
                        } else {
                            div { class: "rounded-2xl border border-dashed border-border bg-card px-4 py-8 text-center text-sm text-muted-foreground",
                                "No keys"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn provider_default_base_url(kind: &str) -> Option<&'static str> {
    match kind {
        "openai" => Some("https://api.openai.com/v1"),
        "deepseek" => Some("https://api.deepseek.com"),
        "siliconflow" => Some("https://api.siliconflow.cn/v1"),
        "dashscope" => Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
        "doubao" => Some("https://ark.cn-beijing.volces.com/api/v3"),
        _ => None,
    }
}

#[component]
fn ModelEditorDialog(
    user_id: String,
    open: bool,
    session: u64,
    mode: ModelEditorMode,
    mut draft: Signal<ModelEditorDraft>,
    api_keys: Vec<user::UserAIKeyDTO>,
    on_open_change: EventHandler<bool>,
    on_saved: EventHandler<user::UserAIConfigDTO>,
    on_message: EventHandler<String>,
) -> Element {
    let mut saving = use_signal(|| false);
    let mut provider_api_key = use_signal(String::new);
    let current = draft();
    let info = model_type_info(&current.capability);
    let is_edit = mode.is_edit();
    let selected_api_key_ref = current.api_key_ref.clone();
    let has_saved_api_keys = !api_keys.is_empty();
    let user_id_for_save = user_id.clone();
    let on_message_for_save = on_message.clone();
    let on_saved_for_save = on_saved.clone();
    let on_open_change_for_save = on_open_change.clone();

    let save_model = move |_| {
        let request_user_id = user_id_for_save.clone();
        let snapshot = draft();
        let api_key_input = provider_api_key();
        async move {
            saving.set(true);
            on_message_for_save.call(String::new());
            let resolved_api_key_ref = if !has_saved_api_keys && !api_key_input.trim().is_empty() {
                let generated_name = if snapshot.provider_name.trim().is_empty() {
                    format!("{} key", provider_kind_label(&snapshot.provider_kind))
                } else {
                    format!("{} key", snapshot.provider_name.trim())
                };
                match user::save_user_ai_key(
                    request_user_id.clone(),
                    user::SaveUserAIKeyInput {
                        id: None,
                        name: generated_name,
                        api_key: Some(api_key_input),
                    },
                )
                .await
                {
                    Ok(after_key) => pick_saved_api_key_ref(
                        &after_key,
                        &snapshot.provider_name,
                        &snapshot.provider_kind,
                    ),
                    Err(err) => {
                        on_message_for_save.call(format!("保存 API key 失败: {err}"));
                        saving.set(false);
                        return;
                    }
                }
            } else {
                Some(snapshot.api_key_ref.clone()).filter(|value| !value.trim().is_empty())
            };
            let provider_id_for_save = if is_edit {
                snapshot.provider_id.clone()
            } else {
                String::new()
            };
            let route_id_for_save = if is_edit {
                snapshot.route_id.clone()
            } else {
                String::new()
            };

            let provider_input = user::SaveUserAIProviderInput {
                id: Some(provider_id_for_save.clone()).filter(|value| !value.trim().is_empty()),
                kind: snapshot.provider_kind.clone(),
                name: snapshot.provider_name.clone(),
                base_url: snapshot.provider_base_url.clone(),
                api_key_ref: resolved_api_key_ref,
                enabled: snapshot.provider_enabled,
            };
            let provider_kind_value = provider_input.kind.clone();
            let provider_name_value = provider_input.name.clone();
            let provider_base_url_value = provider_input.base_url.clone();

            match user::save_user_ai_provider(request_user_id.clone(), provider_input).await {
                Ok(after_provider) => {
                    let next_provider_id = pick_saved_provider_id(
                        &after_provider,
                        &provider_id_for_save,
                        &provider_kind_value,
                        &provider_name_value,
                        &provider_base_url_value,
                    );

                    if next_provider_id.trim().is_empty() {
                        on_message_for_save.call("保存失败：未找到刚保存的供应商".to_string());
                    } else {
                        let ndims = if snapshot.capability == "embedding" {
                            Some(FIXED_EMBEDDING_NDIMS)
                        } else {
                            None
                        };
                        let route_input = user::SaveUserAIRouteInput {
                            id: Some(route_id_for_save).filter(|value| !value.trim().is_empty()),
                            capability: snapshot.capability.clone(),
                            provider_id: next_provider_id,
                            model: snapshot.route_model.clone(),
                            embedding_ndims: ndims,
                            enabled: snapshot.route_enabled,
                        };

                        match user::save_user_ai_route(request_user_id, route_input).await {
                            Ok(next) => {
                                on_saved_for_save.call(next);
                                on_open_change_for_save.call(false);
                                on_message_for_save.call("模型配置已保存".to_string());
                            }
                            Err(err) => {
                                on_saved_for_save.call(after_provider);
                                on_message_for_save.call(format!("保存模型路由失败: {err}"));
                            }
                        }
                    }
                }
                Err(err) => on_message_for_save.call(format!("保存供应商失败: {err}")),
            }

            saving.set(false);
        }
    };

    rsx! {
        DialogRoot {
            open,
            on_open_change: move |next| on_open_change.call(next),
            DialogContent { class: "max-h-[min(86dvh,760px)] w-[calc(100vw-1rem)] max-w-[760px] overflow-hidden rounded-2xl border border-border bg-card p-0 text-left shadow-lg sm:w-[calc(100vw-2rem)]",
                div { class: "flex min-h-0 max-h-[min(86dvh,760px)] flex-col",
                    div { class: "relative shrink-0 border-b border-border/50 px-6 py-5 text-center",
                        div { class: "mx-auto w-full",
                            DialogTitle { class: "text-lg font-medium tracking-tight",
                                if is_edit { "编辑模型" } else { "添加模型" }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "absolute right-5 top-1/2 -translate-y-1/2 rounded-full border border-border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                            onclick: move |_| on_open_change.call(false),
                            X { size: 16 }
                        }
                    }
                    div { class: "min-h-0 flex-1 space-y-4 overflow-y-auto px-4 py-5 md:px-6",
                        div { class: "space-y-4",
                            label { class: "flex flex-col gap-2",
                                span { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                    Server { size: 16 }
                                    "Name"
                                }
                                Input {
                                    class: "rounded-md border border-border bg-card px-3 py-2.5 text-sm shadow-none transition-all hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
                                    value: current.provider_name.clone(),
                                    placeholder: "Provider name",
                                    oninput: move |e: FormEvent| {
                                        draft.with_mut(|next| next.provider_name = e.value());
                                    },
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                span { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                    Server { size: 16 }
                                    "Provider kind"
                                }
                                Select::<String> {
                                    key: "provider-kind-select:{session}:{current.provider_id}:{current.route_id}",
                                    default_value: current.provider_kind.clone(),
                                    on_value_change: move |kind: Option<String>| {
                                        if let Some(kind) = kind {
                                            draft.with_mut(|next| {
                                                let previous_kind = next.provider_kind.clone();
                                                let previous_name = next.provider_name.clone();
                                                next.provider_kind = kind.clone();
                                                if let Some(default_url) = provider_default_base_url(&kind) {
                                                    next.provider_base_url = default_url.to_string();
                                                }
                                                if previous_name.trim().is_empty()
                                                    || previous_name.trim() == provider_kind_label(&previous_kind)
                                                {
                                                    next.provider_name = provider_kind_label(&kind).to_string();
                                                }
                                            });
                                        }
                                    },
                                    for (index, option) in PROVIDER_KIND_OPTIONS.iter().enumerate() {
                                        SelectOption::<String> {
                                            key: "{option.value}",
                                            index,
                                            value: option.value.to_string(),
                                            text_value: option.label.to_string(),
                                            "{option.label}"
                                        }
                                    }
                                }
                            }
                            if api_keys.is_empty() {
                                label { class: "flex flex-col gap-2",
                                    span { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                        KeyRound { size: 16 }
                                        "API key"
                                    }
                                    Input {
                                        class: "rounded-md border border-border bg-card px-3 py-2.5 text-sm shadow-none transition-all hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
                                        value: provider_api_key(),
                                        placeholder: "sk-...",
                                        oninput: move |e: FormEvent| provider_api_key.set(e.value()),
                                    }
                                }
                            } else {
                                div { class: "flex flex-col gap-2",
                                    label { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                        KeyRound { size: 16 }
                                        "API key"
                                    }
                                    Select::<String> {
                                        key: "api-key-select:{session}:{current.provider_id}:{current.route_id}:{selected_api_key_ref}",
                                        default_value: selected_api_key_ref.clone(),
                                        on_value_change: move |secret_ref: Option<String>| {
                                            draft.with_mut(|next| {
                                                next.api_key_ref = secret_ref.unwrap_or_default();
                                            });
                                        },
                                        SelectOption::<String> {
                                            index: 0usize,
                                            value: String::new(),
                                            text_value: "未选择".to_string(),
                                            "未选择"
                                        }
                                        for (index, item) in api_keys.clone().into_iter().enumerate() {
                                            SelectOption::<String> {
                                                key: "{item.id}",
                                                index: index + 1,
                                                value: item.secret_ref.clone(),
                                                text_value: item.name.clone(),
                                                "{item.name}"
                                            }
                                        }
                                    }
                                }
                            }
                            label { class: "flex flex-col gap-2",
                                span { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                    Route { size: 16 }
                                    "Base URL"
                                }
                                Input {
                                    class: "rounded-md border border-border bg-card px-3 py-2.5 text-sm shadow-none transition-all hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
                                    value: current.provider_base_url.clone(),
                                    placeholder: "https://api.example.com/v1",
                                    oninput: move |e: FormEvent| {
                                        draft.with_mut(|next| next.provider_base_url = e.value());
                                    },
                                }
                            }
                            label { class: "flex flex-col gap-2",
                                span { class: "flex items-center gap-2 text-sm font-medium text-foreground",
                                    BrainCircuit { size: 16 }
                                    "Model"
                                }
                                Input {
                                    class: "rounded-md border border-border bg-card px-3 py-2.5 text-sm shadow-none transition-all hover:border-foreground/20 focus:border-foreground/40 focus:ring-4 focus:ring-foreground/5",
                                    value: current.route_model.clone(),
                                    placeholder: info.model_placeholder,
                                    oninput: move |e: FormEvent| {
                                        draft.with_mut(|next| next.route_model = e.value());
                                    },
                                }
                            }
                        }
                        div { class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                            SwitcherLine {
                                label: "启用供应商".to_string(),
                                enabled: current.provider_enabled,
                                on_change: move |next: bool| {
                                    draft.with_mut(|item| item.provider_enabled = next);
                                },
                            }
                            SwitcherLine {
                                label: "设为当前启用".to_string(),
                                enabled: current.route_enabled,
                                on_change: move |next: bool| {
                                    draft.with_mut(|item| item.route_enabled = next);
                                },
                            }
                        }
                    }
                    div { class: "flex shrink-0 flex-col gap-3 border-t border-border/50 px-6 py-5 sm:flex-row sm:justify-end",
                        div { class: "flex flex-col gap-3 sm:flex-row sm:justify-end",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "rounded-md border border-border px-5 py-2 transition-colors hover:border-foreground/30 hover:bg-card text-foreground",
                            onclick: move |_| on_open_change.call(false),
                            "取消"
                        }
                        Button {
                            class: "rounded-md bg-foreground px-5 py-2 text-background shadow-xs transition-all hover:opacity-90",
                            disabled: saving(),
                            onclick: save_model,
                            Save { size: 16 }
                            if saving() { "保存中..." } else if is_edit { "保存修改" } else { "添加模型" }
                        }
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
    let has_config = status.is_some();
    let enabled = status.as_ref().map(|item| item.enabled).unwrap_or(false);

    let state_text = if enabled {
        "已启用"
    } else if has_config {
        "已配置 (未启用)"
    } else {
        "未配置"
    };

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
                "group relative min-h-44 rounded-2xl border p-5 text-left transition-all duration-300 {}",
                if enabled {
                    "border-foreground/20 bg-foreground/[0.02] text-foreground shadow-sm"
                } else if has_config {
                    "border-border bg-card text-foreground hover:border-foreground/30 hover:bg-card/80 opacity-80"
                } else {
                    "border-border border-dashed bg-card/50 text-foreground hover:border-foreground/30 hover:border-solid hover:bg-card/80 opacity-60 hover:opacity-100"
                }
            ),
            onclick: move |_| onclick.call(info.capability.to_string()),
            div { class: "flex items-start justify-between gap-3",
                div { class: format!(
                    "flex h-10 w-10 items-center justify-center rounded-full transition-colors {}",
                    if enabled {
                        "bg-foreground text-background shadow-xs"
                    } else if has_config {
                        "bg-foreground/10 text-foreground"
                    } else {
                        "bg-muted text-muted-foreground group-hover:bg-foreground/10 group-hover:text-foreground"
                    }
                ),
                    TypeIcon { capability: info.capability.to_string(), size: 18 }
                }
                span { class: format!(
                    "rounded-md border px-2 py-0.5 text-[11px] font-medium transition-colors {}",
                    if enabled {
                        "border-foreground/20 bg-foreground/5 text-foreground"
                    } else if has_config {
                        "border-border bg-card text-muted-foreground"
                    } else {
                        "border-transparent text-muted-foreground/60"
                    }
                ), "{state_text}" }
            }
            div { class: "mt-5 text-xs font-medium uppercase tracking-widest text-muted-foreground", "{info.current_label}" }
            div { class: "mt-2 text-xl font-medium tracking-tight text-foreground", "{info.title}" }
            div { class: "mt-1.5 line-clamp-2 text-sm leading-relaxed text-muted-foreground", "{info.subtitle}" }
            div { class: "mt-4 truncate text-xs text-muted-foreground",
                if has_config {
                    "{reason} · {model}"
                } else {
                    "{info.subtitle}"
                }
            }
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
    onedit: EventHandler<ModelRoutePick>,
    ondelete: EventHandler<String>,
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
    let delete_route_id = route.id.clone();
    let mut delete_confirm_open = use_signal(|| false);

    rsx! {
        div { class: "group w-full rounded-xl border border-border bg-card p-4 transition-all hover:border-foreground/30 hover:bg-card/80 hover:shadow-sm",
            div { class: "flex items-start gap-3",
                button {
                    r#type: "button",
                    class: "min-w-0 flex-1 text-left",
                    onclick: move |_| onedit.call(ModelRoutePick { route: click_route.clone(), provider: click_provider.clone() }),
                    div { class: "flex min-w-0 flex-col gap-3 sm:flex-row sm:items-start sm:justify-between",
                        div { class: "min-w-0",
                            div { class: "flex items-center gap-2 text-[15px] font-medium tracking-tight text-foreground",
                                TypeIcon { capability: route.capability.clone(), size: 16 }
                                span { class: "truncate", "{route.model}" }
                            }
                            div { class: "mt-1.5 truncate text-sm text-muted-foreground", "{provider_name} · {provider_kind}" }
                            div { class: "mt-1.5 text-[13px] text-muted-foreground/80", "{key_state}" }
                        }
                        div { class: "flex shrink-0 items-center gap-3",
                            span { class: format!(
                                "rounded-md border px-2 py-0.5 text-[11px] font-medium {}",
                                if route.enabled { "border-foreground/20 bg-foreground/5 text-foreground" } else { "border-transparent bg-muted text-muted-foreground" }
                            ),
                                if route.enabled { "Active" } else { "Disabled" }
                            }
                            span { class: "inline-flex h-8 w-8 items-center justify-center rounded-full border border-transparent text-muted-foreground transition-colors group-hover:border-border group-hover:bg-background",
                                Pencil { size: 14 }
                            }
                        }
                    }
                }
                PopoverRoot {
                    open: delete_confirm_open(),
                    on_open_change: move |open| delete_confirm_open.set(open),
                    PopoverTrigger {
                        class: "!h-8 !w-8 !rounded-full !border !border-transparent !bg-transparent !p-0 !text-muted-foreground transition-colors hover:!border-border hover:!bg-destructive/10 hover:!text-destructive group-hover:!border-border",
                        Trash2 { size: 14 }
                    }
                    PopoverContent {
                        class: "w-56 rounded-xl border border-border bg-card p-4 shadow-lg".to_string(),
                        div { class: "text-sm leading-relaxed text-foreground",
                            "确认删除该模型配置吗？"
                        }
                        div { class: "mt-1 text-xs text-muted-foreground", "删除后不可恢复。" }
                        div { class: "mt-4 flex items-center justify-end gap-2",
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                class: "rounded-md border border-border px-3 transition-colors hover:bg-muted text-foreground",
                                onclick: move |_| delete_confirm_open.set(false),
                                "取消"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                class: "rounded-md bg-destructive px-3 text-destructive-foreground shadow-sm transition-opacity hover:opacity-90",
                                onclick: move |_| {
                                    delete_confirm_open.set(false);
                                    ondelete.call(delete_route_id.clone());
                                },
                                "删除"
                            }
                        }
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
            class: "group w-full rounded-2xl border border-dashed border-border bg-card px-6 py-10 text-left transition-all hover:border-foreground/30 hover:bg-card/80",
            onclick: move |event| onclick.call(event),
            div { class: "mb-4 flex h-12 w-12 items-center justify-center rounded-full border border-border bg-background text-muted-foreground transition-colors group-hover:border-foreground/20 group-hover:text-foreground",
                Plus { size: 20 }
            }
            div { class: "text-base font-medium tracking-tight text-foreground", "添加第一个{title}" }
        }
    }
}

#[component]
fn SwitcherLine(label: String, enabled: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "flex w-full items-center justify-between rounded-xl border border-border bg-card px-5 py-4 text-left transition-colors hover:border-foreground/20",
            div { class: "min-w-0",
                div { class: "text-sm font-medium text-foreground", "{label}" }
            }
            div { class: "flex items-center gap-3",
                span { class: "text-xs font-medium text-muted-foreground", if enabled { "ON" } else { "OFF" } }
                Switch {
                    checked: enabled,
                    on_checked_change: move |next: bool| on_change.call(next),
                }
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

fn new_editor_draft(info: ModelTypeInfo) -> ModelEditorDraft {
    ModelEditorDraft {
        capability: info.capability.to_string(),
        provider_id: String::new(),
        provider_kind: info.default_kind.to_string(),
        provider_name: info.default_name.to_string(),
        provider_base_url: provider_default_base_url(info.default_kind)
            .unwrap_or(info.default_base_url)
            .to_string(),
        api_key_ref: String::new(),
        provider_enabled: true,
        route_id: String::new(),
        route_model: String::new(),
        route_enabled: true,
    }
}

fn editor_draft_from_pick(picked: &ModelRoutePick) -> ModelEditorDraft {
    let info = model_type_info(&picked.route.capability);
    let provider_kind = picked
        .provider
        .as_ref()
        .map(|provider| provider.kind.as_str())
        .unwrap_or(info.default_kind);
    let provider_base_url = picked
        .provider
        .as_ref()
        .map(|provider| provider.base_url.clone())
        .unwrap_or_else(|| {
            provider_default_base_url(provider_kind)
                .unwrap_or(info.default_base_url)
                .to_string()
        });

    ModelEditorDraft {
        capability: picked.route.capability.clone(),
        provider_id: picked
            .provider
            .as_ref()
            .map(|provider| provider.id.clone())
            .unwrap_or_else(|| picked.route.provider_id.clone()),
        provider_kind: provider_kind.to_string(),
        provider_name: picked
            .provider
            .as_ref()
            .map(|provider| provider.name.clone())
            .unwrap_or_else(|| info.default_name.to_string()),
        provider_base_url,
        api_key_ref: picked
            .provider
            .as_ref()
            .and_then(|provider| provider.api_key_ref.clone())
            .unwrap_or_default(),
        provider_enabled: picked
            .provider
            .as_ref()
            .map(|provider| provider.enabled)
            .unwrap_or(true),
        route_id: picked.route.id.clone(),
        route_model: picked.route.model.clone(),
        route_enabled: picked.route.enabled,
    }
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
        "dashscope" => "DashScope",
        "doubao" => "Doubao",
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

fn pick_saved_api_key_ref(
    config: &user::UserAIConfigDTO,
    provider_name: &str,
    provider_kind: &str,
) -> Option<String> {
    let generated_name = if provider_name.trim().is_empty() {
        format!("{} key", provider_kind_label(provider_kind))
    } else {
        format!("{} key", provider_name.trim())
    };

    config
        .api_keys
        .iter()
        .filter(|item| item.name == generated_name)
        .max_by(|left, right| left.updated_at.cmp(&right.updated_at))
        .map(|item| item.secret_ref.clone())
}
