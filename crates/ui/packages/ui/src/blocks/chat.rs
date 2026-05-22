use dioxus::prelude::*;
use dioxus_icons::lucide::{Send, Sparkles};
use chrono::Utc;
use futures_util::StreamExt;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::input::Input;
use crate::components::ui::skeleton::Skeleton;
use crate::views::FIRST_MSG;
use api::chat;

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub text: String,
    pub is_bot: bool,
    pub is_skeleton: bool,
}

pub static CHAT_MESSAGES: GlobalSignal<Vec<ChatMessage>> = Signal::global(Vec::new);
pub static ACTIVE_SESSION_ID: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CHAT_INPUT: GlobalSignal<String> = Signal::global(String::new);
pub static CHAT_NEXT_ID: GlobalSignal<u64> = Signal::global(|| 1_u64);

#[component]
pub fn ChatBlock(session_id: Option<String>) -> Element {
    let session_id_effect = session_id.clone();
    let session_id_resource = session_id.clone();
    let session_id_send = session_id.clone();

    let mut target_session_id = use_signal(|| String::new());

    use_effect(move || {
        let sid = match &session_id_effect {
            Some(id) => id.clone(),
            None => Utc::now().format("%Y-%m-%d").to_string(),
        };
        target_session_id.set(sid);
    });

    let _history_loader = use_resource(move || {
        let session_id_opt = session_id_resource.clone();
        async move {
            let sid = target_session_id();
            if sid.is_empty() {
                return;
            }

            if ACTIVE_SESSION_ID.read().as_ref() == Some(&sid) && !CHAT_MESSAGES.read().is_empty() {
                return;
            }

            *ACTIVE_SESSION_ID.write() = Some(sid.clone());

            match chat::get_session_history(sid.clone()).await {
                Ok(history) => {
                    if history.is_empty() {
                        if session_id_opt.is_some() {
                            *CHAT_MESSAGES.write() = vec![ChatMessage {
                                id: 0,
                                text: "嗨！我是 warmmy，你的对话饮食助理。今天有什么想记录的，或者关于饮食健康的疑问吗？🍎".to_string(),
                                is_bot: true,
                                is_skeleton: false,
                            }];
                        } else {
                            *CHAT_MESSAGES.write() = Vec::new();
                        }
                        *CHAT_NEXT_ID.write() = 1;
                    } else {
                        if session_id_opt.is_none() {
                            navigator().replace(format!("/{}", sid));
                        }
                        let mut loaded_msgs = Vec::new();
                        let mut current_next_id = 1_u64;
                        for msg in history {
                            loaded_msgs.push(ChatMessage {
                                id: current_next_id,
                                text: msg.content,
                                is_bot: msg.role != "user",
                                is_skeleton: false,
                            });
                            current_next_id += 1;
                        }
                        *CHAT_MESSAGES.write() = loaded_msgs;
                        *CHAT_NEXT_ID.write() = current_next_id;
                    }
                }
                Err(_) => {
                    *CHAT_MESSAGES.write() = vec![ChatMessage {
                        id: 0,
                        text: "嗨！今天想吃点什么呢？".to_string(),
                        is_bot: true,
                        is_skeleton: false,
                    }];
                    *CHAT_NEXT_ID.write() = 1;
                }
            }
        }
    });

    let execute_send = std::rc::Rc::new(move |content: String| {
        let user_id = CHAT_NEXT_ID();
        *CHAT_NEXT_ID.write() = user_id + 1;
        CHAT_MESSAGES.write().push(ChatMessage {
            id: user_id,
            text: content.clone(),
            is_bot: false,
            is_skeleton: false,
        });

        let bot_id = CHAT_NEXT_ID();
        *CHAT_NEXT_ID.write() = bot_id + 1;
        CHAT_MESSAGES.write().push(ChatMessage {
            id: bot_id,
            text: String::new(),
            is_bot: true,
            is_skeleton: true,
        });

        let content_for_server = content;
        let sid = target_session_id();

        *ACTIVE_SESSION_ID.write() = Some(sid.clone());

        if session_id_send.is_none() {
            navigator().replace(format!("/{}", sid));
        }

        spawn(async move {
            match chat::echo_stream(content_for_server.clone(), sid.clone()).await {
                Ok(mut stream) => {
                    let mut first = true;
                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(text) => {
                                if text.is_empty() {
                                    continue;
                                }
                                let mut all = CHAT_MESSAGES.write();
                                if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                                    if first {
                                        bot_msg.is_skeleton = false;
                                        first = false;
                                    }
                                    bot_msg.text.push_str(&text);
                                }
                            }
                            Err(err) => {
                                let mut all = CHAT_MESSAGES.write();
                                if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                                    bot_msg.is_skeleton = false;
                                    bot_msg.text.push_str(&format!("\n[stream error] {err}"));
                                }
                                return;
                            }
                        }
                    }
                }
                Err(stream_err) => {
                    let fallback = match chat::echo(content_for_server, sid).await {
                        Ok(resp) => resp.reply,
                        Err(err) => format!("Server error: {stream_err}; fallback failed: {err}"),
                    };
                    let mut all = CHAT_MESSAGES.write();
                    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
                        bot_msg.is_skeleton = false;
                        bot_msg.text = fallback;
                    }
                }
            }
        });
    });

    let execute_send_effect = execute_send.clone();
    use_effect(move || {
        let first_msg_opt = FIRST_MSG.read().clone();
        if let Some(content) = first_msg_opt {
            *FIRST_MSG.write() = None;
            execute_send_effect(content);
        }
    });

    let execute_send_input = execute_send.clone();
    let send_message = std::rc::Rc::new(move || {
        let content = CHAT_INPUT().trim().to_string();
        if content.is_empty() {
            return;
        }
        *CHAT_INPUT.write() = String::new();
        execute_send_input(content);
    });

    let send_message_keydown = send_message.clone();
    let send_message_click = send_message.clone();

    rsx! {
        Card {
            class: "flex flex-col h-full min-h-0 max-w-3xl mx-auto border-x border-border/50 bg-card/30",

            div {
                class: "flex items-center py-4 px-6 border-b border-border sticky top-0 bg-background/80 backdrop-blur z-10",
                h2 {
                    class: "font-doodle text-2xl font-bold flex items-center gap-2 text-foreground",
                    Sparkles { size: 24, class: "text-[#FFAD1A]" }
                    "Diet Buddy"
                }
            }

            div {
                class: "flex-1 min-h-0 overflow-y-auto space-y-6 pt-6 pb-6 px-6 hide-scrollbar",
                for msg in CHAT_MESSAGES().iter() {
                    div {
                        key: "{msg.id}",
                        class: "max-w-[80%] md:max-w-[70%] rounded-[24px] p-4 font-medium text-[15px] leading-relaxed shadow-sm",
                        class: if msg.is_bot {
                            "bg-muted border border-border text-foreground self-start rounded-tl-sm"
                        } else {
                            "bg-foreground text-background self-end ml-auto rounded-tr-sm"
                        },
                        if msg.is_skeleton {
                            div {
                                class: "flex flex-col gap-2 w-48",
                                Skeleton { class: "h-4 w-full rounded" }
                                Skeleton { class: "h-4 w-[80%] rounded" }
                            }
                        } else {
                            "{msg.text}"
                        }
                    }
                }
            }

            div {
                class: "w-full px-6 pt-4 pb-6 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80",
                div {
                    class: "relative flex items-center bg-card rounded-[24px] border border-border shadow-sm p-1.5 focus-within:ring-2 ring-[#FFAD1A]/20 transition-all",
                    Input {
                        class: "flex-1 bg-transparent px-4 py-2 outline-none font-medium text-foreground placeholder:text-muted-foreground border-none shadow-none",
                        r#type: "text",
                        placeholder: "Type your meal...",
                        value: CHAT_INPUT(),
                        oninput: move |e: FormEvent| *CHAT_INPUT.write() = e.value(),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                send_message_keydown();
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        class: "bg-foreground p-3 rounded-full text-background hover:opacity-90 transition-opacity shadow-sm",
                        onclick: move |_| send_message_click(),
                        Send { size: 20, class: "ml-0.5" }
                    }
                }
            }
        }
    }
}
