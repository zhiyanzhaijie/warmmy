use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::CompletionClient;
use rig::completion::Chat;
use rig::message::Message;
use rig::message::ToolChoice;
use rig::providers::{deepseek, openai};
use rig::streaming::{StreamedAssistantContent, StreamingChat};

use app::app_error::{AppError, AppResult};
use app::conversation::{
    ChatMessageRepositoryPort, ConversationReplyStream, SendUserMessageCommand,
    SendUserMessageResult, SendUserMessageUseCase,
};
use app::meal::MealRecordRepositoryPort;
use app::user::UserProfileRepositoryPort;
use domain::UserId;

use crate::agent::guardrail::{GuardrailDecision, GuardrailHook};
use crate::agent::prompt::conversation_preamble;
use crate::agent::tools::meal_log::MealLogTool;

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
    guardrail: GuardrailHook,
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
            guardrail: GuardrailHook,
        }
    }

    async fn load_rig_history(&self, user_id: &UserId, session_id: &str) -> Vec<Message> {
        let raw = self
            .repo
            .find_by_session(user_id, session_id)
            .await
            .unwrap_or_default();

        raw.into_iter()
            .map(|msg| {
                if msg.role == "user" {
                    Message::user(msg.content.as_str())
                } else {
                    Message::assistant(msg.content.as_str())
                }
            })
            .collect()
    }

    fn build_tool(&self, user_id: &UserId) -> MealLogTool {
        MealLogTool::new(
            user_id.clone(),
            self.user_profiles.clone(),
            self.meals.clone(),
        )
    }
}

#[async_trait]
impl SendUserMessageUseCase for ConversationAgent {
    async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult> {
        if let GuardrailDecision::Reject(reason) = self.guardrail.check_input(&command.content) {
            return Err(AppError::validation(reason));
        }

        let user_id = command.user_id.as_str();
        let session_id = command.session_id.as_str();
        let history = self.load_rig_history(&command.user_id, session_id).await;

        let tool = self.build_tool(&command.user_id);
        let prompt = &command.content;
        let model = self.config.model.as_str();
        let preamble = conversation_preamble();

        let reply = match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(4)
                    .tool(tool)
                    .build()
                    .chat(prompt, history)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))?
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(4)
                    .tool(tool)
                    .build()
                    .chat(prompt, history)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))?
            }
            p => return Err(AppError::internal(format!("unsupported provider: {p}"))),
        };

        let _ = self
            .repo
            .save_message(&command.user_id, session_id, "user", &command.content)
            .await;
        let _ = self
            .repo
            .save_message(&command.user_id, session_id, "assistant", &reply)
            .await;

        Ok(SendUserMessageResult {
            reply,
            session_id: command.session_id,
        })
    }

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        if let GuardrailDecision::Reject(reason) = self.guardrail.check_input(&command.content) {
            return Err(AppError::validation(reason));
        }

        let tool = self.build_tool(&command.user_id);
        let prompt = command.content.clone();
        let preamble = conversation_preamble();
        let model = self.config.model.as_str();
        let user_content = command.content;
        let user_id = command.user_id.clone();
        let session_id_str = command.session_id.clone();

        let history = self.load_rig_history(&command.user_id, &session_id_str).await;
        let repo = self.repo.clone();

        match self.config.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let mut raw = client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(4)
                    .tool(tool)
                    .build()
                    .stream_chat(prompt.as_str(), history)
                    .await;
                let s = async_stream::stream! {
                    let mut has_text_delta = false;
                    let mut full_reply = String::new();
                    while let Some(item) = raw.next().await {
                        match item {
                            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                                if !text.text.is_empty() {
                                    has_text_delta = true;
                                    full_reply.push_str(&text.text);
                                    yield Ok(text.text);
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    full_reply.push_str(r.response());
                                    yield Ok(r.response().to_string());
                                }
                            }
                            Ok(_) => {}
                            Err(e) => { yield Err(AppError::upstream(e.to_string())); break; }
                        }
                    }
                    let _ = repo.save_message(&user_id, &session_id_str, "user", &user_content).await;
                    let _ = repo.save_message(&user_id, &session_id_str, "assistant", &full_reply).await;
                };
                Ok(Box::pin(s))
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.config.api_key)
                    .base_url(&self.config.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                let mut raw = client
                    .agent(model)
                    .preamble(preamble)
                    .tool_choice(ToolChoice::Auto)
                    .default_max_turns(4)
                    .tool(tool)
                    .build()
                    .stream_chat(prompt.as_str(), history)
                    .await;
                let s = async_stream::stream! {
                    let mut has_text_delta = false;
                    let mut full_reply = String::new();
                    while let Some(item) = raw.next().await {
                        match item {
                            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                                if !text.text.is_empty() {
                                    has_text_delta = true;
                                    full_reply.push_str(&text.text);
                                    yield Ok(text.text);
                                }
                            }
                            Ok(MultiTurnStreamItem::FinalResponse(r)) => {
                                if !has_text_delta && !r.response().is_empty() {
                                    full_reply.push_str(r.response());
                                    yield Ok(r.response().to_string());
                                }
                            }
                            Ok(_) => {}
                            Err(e) => { yield Err(AppError::upstream(e.to_string())); break; }
                        }
                    }
                    let _ = repo.save_message(&user_id, &session_id_str, "user", &user_content).await;
                    let _ = repo.save_message(&user_id, &session_id_str, "assistant", &full_reply).await;
                };
                Ok(Box::pin(s))
            }
            p => Err(AppError::internal(format!("unsupported provider: {p}"))),
        }
    }
}
