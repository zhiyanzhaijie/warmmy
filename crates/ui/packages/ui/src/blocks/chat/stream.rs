use dioxus::prelude::*;
use serde_json::Value;

use super::state::{
    ChatActivity, ChatActivityKind, ChatContext, ChatMessage, ChatMessageAttachment,
    ComposerImageAttachment,
};
use api::meal;


#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum ChatStreamWireEvent {
    #[serde(rename = "run_started")]
    RunStarted { run_id: String },
    #[serde(rename = "status")]
    Status {
        kind: ChatActivityWireKind,
        label: String,
    },
    #[serde(rename = "tool_started")]
    ToolStarted { tool_name: String, label: String },
    #[serde(rename = "tool_finished")]
    ToolFinished { tool_name: String },
    #[serde(rename = "tool_error")]
    ToolError { tool_name: String, label: String },
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "interaction_requested")]
    InteractionRequested { interaction: AgentInteractionDTO },
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "done")]
    Done,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChatActivityWireKind {
    Thinking,
    ReadingInput,
    CallingModel,
    UsingTool,
    SavingMemory,
    Persisting,
    WaitingUser,
    Cancelling,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct AgentInteractionDTO {
    id: String,
    kind: String,
    payload: Value,
}

enum ChatStreamEvent {
    RunStarted,
    Activity(ChatActivity),
    ClearActivity,
    TextDelta(String),
    InteractionRequested(AgentInteractionDTO),
}

#[derive(Default)]
struct ChatStreamParser {
    buffer: String,
}

impl ChatStreamParser {
    fn parse(&mut self, chunk: &str) -> Vec<ChatStreamEvent> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();

        while let Some(index) = self.buffer.find('\n') {
            let line = self.buffer[..index].trim().to_string();
            self.buffer.drain(..=index);
            if line.is_empty() {
                continue;
            }
            events.extend(self.parse_line(&line));
        }

        events
    }

    fn finish(&mut self) -> Vec<ChatStreamEvent> {
        let line = self.buffer.trim().to_string();
        self.buffer.clear();
        if line.is_empty() {
            return Vec::new();
        }
        self.parse_line(&line)
    }

    fn parse_line(&self, line: &str) -> Vec<ChatStreamEvent> {
        let mut events = Vec::new();
        match serde_json::from_str::<ChatStreamWireEvent>(line) {
            Ok(ChatStreamWireEvent::RunStarted { run_id }) => {
                let _ = run_id;
                events.push(ChatStreamEvent::RunStarted);
            }
            Ok(ChatStreamWireEvent::Status { kind, label }) => {
                events.push(ChatStreamEvent::Activity(ChatActivity {
                    kind: kind.into(),
                    label,
                    tool_name: None,
                }));
            }
            Ok(ChatStreamWireEvent::ToolStarted { tool_name, label }) => {
                events.push(ChatStreamEvent::Activity(ChatActivity {
                    kind: ChatActivityKind::UsingTool,
                    label,
                    tool_name: Some(tool_name),
                }));
            }
            Ok(ChatStreamWireEvent::ToolFinished { tool_name }) => {
                let _ = tool_name;
                events.push(ChatStreamEvent::Activity(ChatActivity {
                    kind: ChatActivityKind::Thinking,
                    label: "我在整理刚才的结果。".to_string(),
                    tool_name: None,
                }));
            }
            Ok(ChatStreamWireEvent::ToolError { tool_name, label }) => {
                events.push(ChatStreamEvent::Activity(ChatActivity {
                    kind: ChatActivityKind::UsingTool,
                    label,
                    tool_name: Some(tool_name),
                }));
            }
            Ok(ChatStreamWireEvent::TextDelta { text }) => {
                events.push(ChatStreamEvent::TextDelta(text));
            }
            Ok(ChatStreamWireEvent::InteractionRequested { interaction }) => {
                events.push(ChatStreamEvent::InteractionRequested(interaction));
            }
            Ok(ChatStreamWireEvent::Cancelled | ChatStreamWireEvent::Done) => {
                events.push(ChatStreamEvent::ClearActivity);
            }
            Err(_) if line.starts_with('{') => {}
            Err(_) => events.push(ChatStreamEvent::TextDelta(line.to_string())),
        }
        events
    }
}

impl From<ChatActivityWireKind> for ChatActivityKind {
    fn from(kind: ChatActivityWireKind) -> Self {
        match kind {
            ChatActivityWireKind::Thinking => Self::Thinking,
            ChatActivityWireKind::ReadingInput => Self::ReadingInput,
            ChatActivityWireKind::CallingModel => Self::CallingModel,
            ChatActivityWireKind::UsingTool => Self::UsingTool,
            ChatActivityWireKind::SavingMemory => Self::SavingMemory,
            ChatActivityWireKind::Persisting => Self::Persisting,
            ChatActivityWireKind::WaitingUser => Self::WaitingUser,
            ChatActivityWireKind::Cancelling => Self::Cancelling,
        }
    }
}

pub fn next_chat_id(chat_state: ChatContext) -> u64 {
    chat_state
        .messages
        .peek()
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

pub fn active_session_id(chat_state: ChatContext) -> String {
    chat_state
        .active_session_id
        .read()
        .clone()
        .unwrap_or_else(crate::today_session_id)
}

pub fn is_active_session(chat_state: ChatContext, session_id: &str) -> bool {
    chat_state
        .active_session_id
        .peek()
        .as_ref()
        .map(|active| active == session_id)
        .unwrap_or(false)
}

pub fn activate_session(mut chat_state: ChatContext, session_id: String) {
    chat_state.active_session_id.set(Some(session_id.clone()));
    let messages = chat_state
        .session_messages
        .peek()
        .get(&session_id)
        .cloned()
        .unwrap_or_default();
    chat_state.messages.set(messages);
    chat_state.next_id.set(next_chat_id(chat_state).max(1));
}

pub fn visible_session_messages(chat_state: ChatContext, session_id: &str) -> Vec<ChatMessage> {
    chat_state
        .session_messages
        .peek()
        .get(session_id)
        .cloned()
        .unwrap_or_default()
}

pub fn set_active_session_messages(
    mut chat_state: ChatContext,
    session_id: String,
    messages: Vec<ChatMessage>,
    next_id: u64,
) {
    chat_state
        .session_messages
        .write()
        .insert(session_id.clone(), messages.clone());
    chat_state.active_session_id.set(Some(session_id));
    chat_state.messages.set(messages);
    chat_state.next_id.set(next_id.max(1));
}

fn sync_visible_session(mut chat_state: ChatContext, session_id: &str) {
    if is_active_session(chat_state, session_id) {
        let messages = visible_session_messages(chat_state, session_id);
        chat_state.messages.set(messages);
        chat_state.next_id.set(next_chat_id(chat_state).max(1));
    }
}

pub fn append_pending_meal_messages(
    mut chat_state: ChatContext,
    session_id: String,
    pending_meals: Vec<meal::PendingMealLogDTO>,
    start_id: u64,
) {
    if pending_meals.is_empty() {
        return;
    }

    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    let mut next_id = start_id.max(
        all.iter()
            .map(|message| message.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
    );
    for pending_meal in pending_meals {
        if all.iter().any(|message| {
            message.pending_meal.as_ref().map(|meal| &meal.id) == Some(&pending_meal.id)
        }) {
            continue;
        }
        all.push(ChatMessage {
            id: next_id,
            text: String::new(),
            is_bot: true,
            is_skeleton: false,
            is_streaming: false,
            attachments: Vec::new(),
            action: None,
            pending_meal: Some(pending_meal),
        });
        next_id += 1;
    }
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
}

pub fn remove_pending_meal_messages(
    mut chat_state: ChatContext,
    session_id: String,
    pending_ids: &[String],
) {
    if pending_ids.is_empty() {
        return;
    }

    let mut all_sessions = chat_state.session_messages.write();
    if let Some(all) = all_sessions.get_mut(&session_id) {
        all.retain(|message| {
            message
                .pending_meal
                .as_ref()
                .map(|pending| !pending_ids.iter().any(|id| id == &pending.id))
                .unwrap_or(true)
        });
    }
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
}

pub fn append_outgoing_message_pair(
    mut chat_state: ChatContext,
    session_id: String,
    content: String,
    attachments: Vec<ComposerImageAttachment>,
) -> u64 {
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    let user_id = (chat_state.next_id)();
    let bot_id = user_id.saturating_add(1);
    chat_state.next_id.set(bot_id.saturating_add(1));
    all.push(ChatMessage {
        id: user_id,
        text: content,
        is_bot: false,
        is_skeleton: false,
        is_streaming: false,
        attachments: composer_attachments_to_message_attachments(attachments),
        action: None,
        pending_meal: None,
    });
    all.push(ChatMessage {
        id: bot_id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        attachments: Vec::new(),
        action: None,
        pending_meal: None,
    });
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
    bot_id
}

pub fn append_bot_text(mut chat_state: ChatContext, session_id: String, text: String) {
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    let id = all
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    all.push(ChatMessage {
        id,
        text,
        is_bot: true,
        is_skeleton: false,
        is_streaming: false,
        attachments: Vec::new(),
        action: None,
        pending_meal: None,
    });
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
}

pub fn append_streaming_bot_slot(mut chat_state: ChatContext, session_id: String) -> u64 {
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    let id = all
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    all.push(ChatMessage {
        id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        attachments: Vec::new(),
        action: None,
        pending_meal: None,
    });
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
    id
}

pub async fn append_agent_stream(
    mut chat_state: ChatContext,
    mut stream: dioxus::fullstack::payloads::TextStream,
    bot_id: u64,
    session_id: String,
) {
    let mut first = true;
    let mut parser = ChatStreamParser::default();
    loop {
        let chunk = stream.next().await;

        let Some(chunk) = chunk else {
            break;
        };

        match chunk {
            Ok(text) => {
                if text.is_empty() {
                    continue;
                }
                for event in parser.parse(&text) {
                    match event {
                        ChatStreamEvent::RunStarted => {}
                        ChatStreamEvent::Activity(activity) => {
                            set_session_activity(chat_state, session_id.clone(), activity);
                        }
                        ChatStreamEvent::ClearActivity => {
                            clear_session_activity(chat_state, &session_id);
                        }
                        ChatStreamEvent::TextDelta(delta) => {
                            let mut all_sessions = chat_state.session_messages.write();
                            let all = all_sessions.entry(session_id.clone()).or_default();
                            let bot_index = ensure_streaming_bot_slot(all, bot_id);
                            let bot_msg = &mut all[bot_index];
                            if first {
                                bot_msg.is_skeleton = false;
                                first = false;
                            }
                            bot_msg.text.push_str(&delta);
                            drop(all_sessions);
                            sync_visible_session(chat_state, &session_id);
                        }
                        ChatStreamEvent::InteractionRequested(interaction) => {
                            set_session_activity(
                                chat_state,
                                session_id.clone(),
                                ChatActivity {
                                    kind: ChatActivityKind::WaitingUser,
                                    label: "我整理好了一条需要你确认的记录。".to_string(),
                                    tool_name: None,
                                },
                            );
                            handle_interaction_requested(
                                chat_state,
                                session_id.clone(),
                                interaction,
                            );
                        }
                    }
                }
            }
            Err(err) => {
                stop_stream_with_text(
                    chat_state,
                    &session_id,
                    bot_id,
                    format!("\n[stream error] {err}"),
                );
                return;
            }
        }
    }

    for event in parser.finish() {
        match event {
            ChatStreamEvent::RunStarted => {}
            ChatStreamEvent::Activity(activity) => {
                set_session_activity(chat_state, session_id.clone(), activity);
            }
            ChatStreamEvent::ClearActivity => {
                clear_session_activity(chat_state, &session_id);
            }
            ChatStreamEvent::TextDelta(delta) => {
                let mut all_sessions = chat_state.session_messages.write();
                let all = all_sessions.entry(session_id.clone()).or_default();
                let bot_index = ensure_streaming_bot_slot(all, bot_id);
                let bot_msg = &mut all[bot_index];
                if first {
                    bot_msg.is_skeleton = false;
                    first = false;
                }
                bot_msg.text.push_str(&delta);
                drop(all_sessions);
                sync_visible_session(chat_state, &session_id);
            }
            ChatStreamEvent::InteractionRequested(interaction) => {
                set_session_activity(
                    chat_state,
                    session_id.clone(),
                    ChatActivity {
                        kind: ChatActivityKind::WaitingUser,
                        label: "我整理好了一条需要你确认的记录。".to_string(),
                        tool_name: None,
                    },
                );
                handle_interaction_requested(chat_state, session_id.clone(), interaction);
            }
        }
    }
    clear_session_activity(chat_state, &session_id);
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    if let Some(bot_msg) = all.iter_mut().find(|msg| msg.id == bot_id) {
        bot_msg.is_skeleton = false;
        bot_msg.is_streaming = false;
        if bot_msg.text.trim().is_empty() {
            bot_msg.text = "模型没有返回内容。请检查当前模型是否支持流式输出，或尝试更换模型配置。"
                .to_string();
        }
    }
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
}

fn stop_stream_with_text(mut chat_state: ChatContext, session_id: &str, bot_id: u64, text: String) {
    clear_session_activity(chat_state, session_id);
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.to_string()).or_default();
    let bot_index = ensure_streaming_bot_slot(all, bot_id);
    let bot_msg = &mut all[bot_index];
    bot_msg.is_skeleton = false;
    bot_msg.is_streaming = false;
    if bot_msg.text.trim().is_empty() {
        bot_msg.text = text;
    } else {
        bot_msg.text.push_str(&text);
    }
    drop(all_sessions);
    sync_visible_session(chat_state, session_id);
}

pub fn set_session_activity(
    mut chat_state: ChatContext,
    session_id: String,
    activity: ChatActivity,
) {
    chat_state
        .session_activities
        .write()
        .insert(session_id, activity);
}

pub fn clear_session_activity(mut chat_state: ChatContext, session_id: &str) {
    chat_state.session_activities.write().remove(session_id);
}

fn ensure_streaming_bot_slot(messages: &mut Vec<ChatMessage>, bot_id: u64) -> usize {
    if let Some(index) = messages.iter().position(|msg| msg.id == bot_id) {
        return index;
    }
    messages.push(ChatMessage {
        id: bot_id,
        text: String::new(),
        is_bot: true,
        is_skeleton: true,
        is_streaming: true,
        attachments: Vec::new(),
        action: None,
        pending_meal: None,
    });
    messages.len().saturating_sub(1)
}

fn handle_interaction_requested(
    chat_state: ChatContext,
    session_id: String,
    interaction: AgentInteractionDTO,
) {
    if interaction.kind == "meal_log_confirmation" {
        match serde_json::from_value::<meal::PendingMealLogDTO>(interaction.payload) {
            Ok(meal) => push_pending_meal_message(chat_state, session_id, meal),
            Err(err) => {
                append_bot_text(chat_state, session_id, format!("无法渲染待确认操作：{err}"))
            }
        }
    } else {
        append_bot_text(
            chat_state,
            session_id,
            format!(
                "收到暂不支持的操作请求：{} ({})",
                interaction.kind, interaction.id
            ),
        );
    }
}

fn push_pending_meal_message(
    mut chat_state: ChatContext,
    session_id: String,
    pending_meal: meal::PendingMealLogDTO,
) {
    let mut all_sessions = chat_state.session_messages.write();
    let all = all_sessions.entry(session_id.clone()).or_default();
    if all
        .iter()
        .any(|message| message.pending_meal.as_ref().map(|meal| &meal.id) == Some(&pending_meal.id))
    {
        return;
    }
    let id = all
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    all.push(ChatMessage {
        id,
        text: String::new(),
        is_bot: true,
        is_skeleton: false,
        is_streaming: false,
        attachments: Vec::new(),
        action: None,
        pending_meal: Some(pending_meal),
    });
    drop(all_sessions);
    sync_visible_session(chat_state, &session_id);
}

fn composer_attachments_to_message_attachments(
    attachments: Vec<ComposerImageAttachment>,
) -> Vec<ChatMessageAttachment> {
    attachments
        .into_iter()
        .map(|attachment| ChatMessageAttachment {
            id: format!("composer:{}", attachment.id),
            kind: "image".to_string(),
            mime_type: attachment.mime_type,
            size_bytes: attachment.size_bytes,
            width: None,
            height: None,
            data_url: Some(attachment.preview_data_url),
            status: "available".to_string(),
        })
        .collect()
}

