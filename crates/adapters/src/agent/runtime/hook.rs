use std::marker::PhantomData;
use std::sync::Arc;

use app::conversation::ConversationStreamEvent;
use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse};
use rig::message::Message;
use tokio::sync::mpsc::UnboundedSender;

use crate::agent::tool::names;

#[derive(Clone)]
pub struct WarmmyPromptHook<M> {
    guardrail: Arc<GuardrailHook>,
    status_sink: Option<AgentStatusSink>,
    _model: PhantomData<fn() -> M>,
}

pub struct GuardrailHook;

#[derive(Clone)]
pub struct AgentStatusSink {
    tx: UnboundedSender<ConversationStreamEvent>,
}

impl AgentStatusSink {
    pub fn new(tx: UnboundedSender<ConversationStreamEvent>) -> Self {
        Self { tx }
    }

    pub fn emit(&self, event: ConversationStreamEvent) {
        let _ = self.tx.send(event);
    }
}

impl GuardrailHook {
    pub fn check_input(&self, _input: &str) -> GuardrailDecision {
        GuardrailDecision::Allow
    }

    pub fn check_output(&self, _output: &str) -> GuardrailDecision {
        GuardrailDecision::Allow
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardrailDecision {
    Allow,
    Reject(String),
}

impl<M> WarmmyPromptHook<M> {
    pub fn new(guardrail: Arc<GuardrailHook>) -> Self {
        Self {
            guardrail,
            status_sink: None,
            _model: PhantomData,
        }
    }

    pub fn with_status_sink(guardrail: Arc<GuardrailHook>, status_sink: AgentStatusSink) -> Self {
        Self {
            guardrail,
            status_sink: Some(status_sink),
            _model: PhantomData,
        }
    }
}

impl<M> PromptHook<M> for WarmmyPromptHook<M>
where
    M: CompletionModel,
{
    async fn on_completion_call(&self, prompt: &Message, _history: &[Message]) -> HookAction {
        if let Some(input) = user_text(prompt) {
            if let GuardrailDecision::Reject(reason) = self.guardrail.check_input(input) {
                return HookAction::terminate(reason);
            }
        }

        HookAction::cont()
    }

    async fn on_completion_response(
        &self,
        _prompt: &Message,
        response: &CompletionResponse<M::Response>,
    ) -> HookAction {
        for content in response.choice.iter() {
            if let rig::message::AssistantContent::Text(text) = content {
                if let GuardrailDecision::Reject(reason) = self.guardrail.check_output(&text.text) {
                    return HookAction::terminate(reason);
                }
            }
        }

        HookAction::cont()
    }

    async fn on_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        internal_call_id: &str,
        args: &str,
    ) -> ToolCallHookAction {
        tracing::info!(
            tool.name = tool_name,
            tool.call_id = tool_call_id,
            tool.internal_call_id = internal_call_id,
            tool.args = args,
            "agent tool call"
        );
        if let Some(status_sink) = &self.status_sink {
            status_sink.emit(ConversationStreamEvent::ToolStarted {
                tool_name: tool_name.to_string(),
                label: tool_status_label(tool_name),
            });
        }

        ToolCallHookAction::cont()
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        internal_call_id: &str,
        args: &str,
        result: &str,
    ) -> HookAction {
        tracing::info!(
            tool.name = tool_name,
            tool.call_id = tool_call_id,
            tool.internal_call_id = internal_call_id,
            tool.args = args,
            tool.result = result,
            "agent tool result"
        );
        if let Some(status_sink) = &self.status_sink {
            status_sink.emit(ConversationStreamEvent::ToolFinished {
                tool_name: tool_name.to_string(),
            });
        }

        HookAction::cont()
    }

    async fn on_text_delta(&self, _text_delta: &str, _aggregated_text: &str) -> HookAction {
        HookAction::cont()
    }
}

fn tool_status_label(tool_name: &str) -> String {
    match tool_name {
        names::PROPOSE_MEAL_LOG => "我在整理这顿饭的待确认记录。",
        names::CONFIRM_MEAL_LOG => "我在把这顿饭正式记下来。",
        names::REJECT_MEAL_LOG => "我在收起这条待确认记录。",
        _ => "我在调用工具处理这件事。",
    }
    .to_string()
}

fn user_text(message: &Message) -> Option<&str> {
    match message {
        Message::User { content } => content.iter().find_map(|content| {
            if let rig::message::UserContent::Text(text) = content {
                Some(text.text.as_str())
            } else {
                None
            }
        }),
        _ => None,
    }
}
