use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::message::ToolChoice;
use rig::providers::{deepseek, openai};
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};

use app::app_error::{AppError, AppResult};
use app::conversation::{
    ChatMessageRepositoryPort, ConversationReplyStream, SendUserMessageCommand,
    SendUserMessageResult, ConversationAgentPort,
};
use app::meal::MealRecordRepositoryPort;
use app::user::UserProfileRepositoryPort;
use domain::UserId;

use crate::agent::guardrail::GuardrailHook;
use crate::agent::hook::WarmmyPromptHook;
use crate::agent::memory::SessionConversationMemory;
use crate::agent::prompt::conversation_preamble;
use crate::agent::tools::meal_log::MealLogTool;

const DEFAULT_MAX_TURNS: usize = 4;
const DEFAULT_HISTORY_WINDOW_MESSAGES: usize = 24;

#[derive(Clone)]
pub struct AgentConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

pub struct ConversationAgent {
    config: AgentConfig,
    meals: Arc<dyn MealRecordRepositoryPort>,
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
    repo: Arc<dyn ChatMessageRepositoryPort>,
    guardrail: Arc<GuardrailHook>,
}

impl ConversationAgent {
    pub fn new(
        config: AgentConfig,
        meals: Arc<dyn MealRecordRepositoryPort>,
        user_profiles: Arc<dyn UserProfileRepositoryPort>,
        repo: Arc<dyn ChatMessageRepositoryPort>,
    ) -> Self {
        Self {
            config,
            meals,
            user_profiles,
            repo,
            guardrail: Arc::new(GuardrailHook),
        }
    }

    fn build_tool(&self, user_id: &UserId) -> MealLogTool {
        MealLogTool::new(
            user_id.clone(),
            self.user_profiles.clone(),
            self.meals.clone(),
        )
    }

    fn build_memory(&self, user_id: &UserId) -> SessionConversationMemory {
        SessionConversationMemory::new(
            user_id.clone(),
            self.repo.clone(),
            DEFAULT_HISTORY_WINDOW_MESSAGES,
        )
    }
}

#[async_trait]
impl ConversationAgentPort for ConversationAgent {
    async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult> {
        let prompt = command.content.as_str();
        let session_id = command.session_id.as_str();
        let model = self.config.model.as_str();
        let preamble = conversation_preamble();
        let tool = self.build_tool(&command.user_id);
        let memory = self.build_memory(&command.user_id);

        let reply = match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(memory)
                    .hook(hook)
                    .tool(tool)
                    .build()
                    .prompt(prompt)
                    .conversation(session_id)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))?
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(memory)
                    .hook(hook)
                    .tool(tool)
                    .build()
                    .prompt(prompt)
                    .conversation(session_id)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))?
            }
            p => return Err(AppError::internal(format!("unsupported provider: {p}"))),
        };

        Ok(SendUserMessageResult {
            reply,
            session_id: command.session_id,
        })
    }

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        let prompt = command.content.clone();
        let session_id = command.session_id.clone();
        let model = self.config.model.as_str();
        let preamble = conversation_preamble();
        let tool = self.build_tool(&command.user_id);
        let memory = self.build_memory(&command.user_id);

        match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                let mut raw = client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(memory)
                    .hook(hook)
                    .tool(tool)
                    .build()
                    .stream_prompt(prompt)
                    .conversation(session_id)
                    .await;
                let s = async_stream::stream! {
                    let mut has_text_delta = false;
                    while let Some(item) = raw.next().await {
                        match item {
                            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                                if !text.text.is_empty() {
                                    has_text_delta = true;
                                    yield Ok(text.text);
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    yield Ok(r.response().to_string());
                                }
                            }
                            Ok(_) => {}
                            Err(e) => { yield Err(AppError::upstream(e.to_string())); break; }
                        }
                    }
                };
                Ok(Box::pin(s))
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let hook = WarmmyPromptHook::new(self.guardrail.clone());
                let mut raw = client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(DEFAULT_MAX_TURNS)
                    .memory(memory)
                    .hook(hook)
                    .tool(tool)
                    .build()
                    .stream_prompt(prompt)
                    .conversation(session_id)
                    .await;
                let s = async_stream::stream! {
                    let mut has_text_delta = false;
                    while let Some(item) = raw.next().await {
                        match item {
                            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                                if !text.text.is_empty() {
                                    has_text_delta = true;
                                    yield Ok(text.text);
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    yield Ok(r.response().to_string());
                                }
                            }
                            Ok(_) => {}
                            Err(e) => { yield Err(AppError::upstream(e.to_string())); break; }
                        }
                    }
                };
                Ok(Box::pin(s))
            }
            p => Err(AppError::internal(format!("unsupported provider: {p}"))),
        }
    }
}
