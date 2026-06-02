use base64::Engine;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ImagePlus, Send, X};
use serde::Deserialize;
use std::rc::Rc;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::textarea::{Textarea, TextareaVariant};

use super::state::{
    ChatContext, ComposerImageAttachment,
};
use super::stream::{active_session_id, append_bot_text};

const MAX_COMPOSER_IMAGE_COUNT: usize = 4;
const MAX_COMPOSER_IMAGE_SIZE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct PickedImages {
    files: Vec<PickedImage>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PickedImage {
    name: String,
    mime_type: String,
    size_bytes: u64,
    data_url: String,
}

#[derive(Clone)]
pub(super) struct SendChatMessage {
    handler: Rc<dyn Fn(String, Vec<ComposerImageAttachment>)>,
}

impl SendChatMessage {
    pub(super) fn new(handler: Rc<dyn Fn(String, Vec<ComposerImageAttachment>)>) -> Self {
        Self { handler }
    }

    fn call(&self, content: String, attachments: Vec<ComposerImageAttachment>) {
        (self.handler)(content, attachments);
    }
}

impl PartialEq for SendChatMessage {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handler, &other.handler)
    }
}

#[component]
pub(super) fn ChatComposer(is_streaming: bool, on_send: SendChatMessage) -> Element {
    let mut chat_state = use_context::<ChatContext>();
    let send_message = {
        let on_send = on_send.clone();
        move || {
            if chat_state
                .messages
                .read()
                .iter()
                .any(|msg| msg.is_streaming || msg.is_skeleton)
            {
                return;
            }
            let content = (chat_state.input)().trim().to_string();
            let attachments = chat_state.composer_attachments.read().clone();
            if content.is_empty() && attachments.is_empty() {
                return;
            }
            chat_state.input.set(String::new());
            chat_state.composer_attachments.write().clear();
            on_send.call(content, attachments);
        }
    };

    let mut send_message_keydown = send_message.clone();
    let mut send_message_click = send_message.clone();
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
            if (window.__warmmyImagePickerHandler) {
                window.removeEventListener("warmmy-images-picked", window.__warmmyImagePickerHandler);
            }

            window.__warmmyImagePickerHandler = (event) => {
                dioxus.send(event.detail || { files: [], error: "missing image picker payload" });
            };

            window.addEventListener("warmmy-images-picked", window.__warmmyImagePickerHandler);

            await new Promise(() => {});
            "#,
        );

        loop {
            match eval.recv::<PickedImages>().await {
                Ok(picked) => append_picked_images(chat_state, picked),
                Err(err) => {
                    append_bot_text(
                        chat_state,
                        active_session_id(chat_state),
                        format!("选择图片失败：{err}"),
                    );
                    return;
                }
            }
        }
    });

    rsx! {
        div {
            class: "px-4 pb-4 pt-2 md:px-5 md:pb-5",
            div {
                class: "rounded-[2rem] border border-border bg-card/85 p-2 shadow-lg backdrop-blur focus-within:shadow-md",
                AttachmentPreviewStrip {}
                div {
                    class: "flex items-end gap-2 rounded-[1.5rem] bg-background/95 p-2",
                    button {
                        r#type: "button",
                        class: "mb-1 inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-full border border-border bg-card text-muted-foreground shadow-sm transition-colors hover:border-foreground/30 hover:bg-muted hover:text-foreground active:scale-[0.98] disabled:opacity-50",
                        disabled: is_streaming,
                        title: "选择图片",
                        onclick: move |_| {
                            pick_images(chat_state);
                        },
                        ImagePlus { size: 16 }
                    }
                    Textarea {
                        variant: TextareaVariant::Ghost,
                        class: "max-h-40 min-h-12 min-w-0 flex-1 resize-none overflow-y-auto border-none bg-transparent px-3 py-3 font-medium leading-relaxed text-foreground shadow-none outline-none placeholder:text-muted-foreground placeholder:whitespace-nowrap placeholder:overflow-hidden placeholder:text-ellipsis [field-sizing:content]",
                        rows: "1",
                        placeholder: "记录餐食，或询问下一顿吃什么...",
                        value: (chat_state.input)(),
                        disabled: is_streaming,
                        oninput: move |e: FormEvent| {
                            chat_state.input.set(e.value());
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter && !e.modifiers().shift() && !is_streaming {
                                send_message_keydown();
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        class: "mb-1 rounded-full bg-foreground p-3 text-background shadow-sm hover:opacity-90",
                        disabled: is_streaming,
                        onclick: move |_| send_message_click(),
                        Send { size: 20, class: "ml-0.5" }
                    }
                }
            }
        }
    }
}

fn pick_images(chat_state: ChatContext) {
    #[cfg(target_os = "android")]
    {
        document::eval(
            r#"
            if (window.WarmmyAndroid && typeof window.WarmmyAndroid.pickImages === "function") {
                window.WarmmyAndroid.pickImages();
            } else {
                window.dispatchEvent(new CustomEvent("warmmy-images-picked", {
                    detail: { files: [], error: "Android image picker bridge is not available" }
                }));
            }
            "#,
        );
        return;
    }

    #[cfg(target_os = "ios")]
    {
        if let Err(err) = crate::platform::pick_images() {
            append_bot_text(
                chat_state,
                active_session_id(chat_state),
                format!("选择图片失败：{err}"),
            );
        }
        return;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        append_bot_text(
            chat_state,
            active_session_id(chat_state),
            "当前平台暂不支持原生图片选择器".to_string(),
        );
    }
}

fn decode_data_url(data_url: &str) -> Result<Vec<u8>, String> {
    let (_, encoded) = data_url
        .split_once(',')
        .ok_or_else(|| "图片数据格式无效".to_string())?;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| err.to_string())
}

fn append_picked_images(mut chat_state: ChatContext, picked: PickedImages) {
    if let Some(err) = picked.error {
        append_bot_text(
            chat_state,
            active_session_id(chat_state),
            format!("选择图片失败：{err}"),
        );
        return;
    }

    let mut appended = Vec::new();
    let existing_count = chat_state.composer_attachments.read().len();
    for file in picked.files {
        if existing_count + appended.len() >= MAX_COMPOSER_IMAGE_COUNT {
            break;
        }
        let mime_type = if file.mime_type.is_empty() {
            "application/octet-stream".to_string()
        } else {
            file.mime_type
        };
        if !mime_type.starts_with("image/") {
            continue;
        }
        if file.size_bytes as usize > MAX_COMPOSER_IMAGE_SIZE_BYTES {
            append_bot_text(
                chat_state,
                active_session_id(chat_state),
                format!("图片 {} 超过大小限制（最多 10MB）", file.name),
            );
            continue;
        }

        match decode_data_url(&file.data_url) {
            Ok(bytes) => {
                let id = (chat_state.attachment_next_id)();
                chat_state.attachment_next_id.set(id.saturating_add(1));
                appended.push(ComposerImageAttachment {
                    id,
                    name: file.name,
                    mime_type,
                    size_bytes: bytes.len() as u64,
                    bytes,
                    preview_data_url: file.data_url,
                });
            }
            Err(err) => {
                append_bot_text(
                    chat_state,
                    active_session_id(chat_state),
                    format!("读取图片失败：{err}"),
                );
            }
        }
    }

    if !appended.is_empty() {
        chat_state.composer_attachments.write().extend(appended);
    }
}

#[component]
fn AttachmentPreviewStrip() -> Element {
    let mut chat_state = use_context::<ChatContext>();
    let mut remove_attachment = move |id: u64| {
        chat_state
            .composer_attachments
            .write()
            .retain(|attachment| attachment.id != id);
    };

    rsx! {
        if !chat_state.composer_attachments.read().is_empty() {
            div { class: "mb-2 flex flex-wrap gap-2 px-2 pt-1",
                for attachment in chat_state.composer_attachments.read().iter() {
                    div {
                        key: "{attachment.id}",
                        class: "group relative h-14 w-14 overflow-hidden rounded-xl border border-border bg-card",
                        img {
                            class: "h-full w-full object-cover",
                            src: attachment.preview_data_url.clone(),
                            alt: attachment.name.clone(),
                        }
                        button {
                            r#type: "button",
                            class: "absolute right-1 top-1 inline-flex h-5 w-5 items-center justify-center rounded-full bg-black/60 text-white opacity-0 transition group-hover:opacity-100",
                            onclick: {
                                let id = attachment.id;
                                move |_| remove_attachment(id)
                            },
                            X { size: 12 }
                        }
                    }
                }
            }
        }
    }
}
