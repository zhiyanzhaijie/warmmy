use std::collections::HashMap;

use app::agents::{TextModelGateway, TextPromptRequest};
use app::app_error::{AppError, AppResult};
use app::conversation::ConversationReplyStream;
use app::user::ResolvedAIModelConfig;
use async_trait::async_trait;
use rig::agent::{Agent, AgentBuilder};
use rig::client::CompletionClient;
use rig::completion::{CompletionModel, Message, Prompt};
use rig::message::ToolChoice;
use rig::providers::{deepseek, openai};
use rig::streaming::StreamingPrompt;
use rig::tool::ToolDyn;

use crate::agent::memory::long_term::retriever::MemoryRetriever;
use crate::agent::memory::SessionConversationMemory;
use crate::agent::runtime::hook::{AgentStatusSink, GuardrailHook, WarmmyPromptHook};
use crate::agent::runtime::rig::{RigConversationRuntime, StreamWrapCtx};

const DEFAULT_MAX_TURNS: usize = 4;

type OpenAiResponsesModel = openai::responses_api::ResponsesCompletionModel;
type OpenAiCompletionsModel = openai::completion::CompletionModel;
type DeepSeekModel = deepseek::CompletionModel;

type WarmmyAgent<M> = Agent<M, WarmmyPromptHook<M>>;
type ConversationAgentBuilder =
    fn(&RigModelFactory, &ResolvedAIModelConfig, AgentSpec) -> AppResult<ConversationAgent>;
type TextPrompter =
    fn(&RigModelFactory, &ResolvedAIModelConfig, &str, String) -> AppResult<TextPrompt>;

#[derive(Clone)]
pub struct RigModelFactory {
    conversation_builders: HashMap<&'static str, ConversationAgentBuilder>,
    text_prompters: HashMap<&'static str, TextPrompter>,
}

impl Default for RigModelFactory {
    fn default() -> Self {
        Self {
            conversation_builders: HashMap::from([
                (
                    "openai",
                    build_openai_responses_agent as ConversationAgentBuilder,
                ),
                (
                    "openai_compatible",
                    build_openai_completions_agent as ConversationAgentBuilder,
                ),
                (
                    "siliconflow",
                    build_openai_completions_agent as ConversationAgentBuilder,
                ),
                (
                    "dashscope",
                    build_openai_completions_agent as ConversationAgentBuilder,
                ),
                (
                    "doubao",
                    build_openai_completions_agent as ConversationAgentBuilder,
                ),
                ("deepseek", build_deepseek_agent as ConversationAgentBuilder),
            ]),
            text_prompters: HashMap::from([
                ("openai", prompt_openai_completions as TextPrompter),
                (
                    "openai_compatible",
                    prompt_openai_completions as TextPrompter,
                ),
                ("siliconflow", prompt_openai_completions as TextPrompter),
                ("dashscope", prompt_openai_completions as TextPrompter),
                ("doubao", prompt_openai_completions as TextPrompter),
                ("deepseek", prompt_deepseek as TextPrompter),
            ]),
        }
    }
}

impl RigModelFactory {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
pub trait ModelGateway: Send + Sync {
    fn conversation_agent(
        &self,
        config: &ResolvedAIModelConfig,
        spec: AgentSpec,
    ) -> AppResult<ConversationAgent>;
}

#[async_trait]
impl ModelGateway for RigModelFactory {
    fn conversation_agent(
        &self,
        config: &ResolvedAIModelConfig,
        spec: AgentSpec,
    ) -> AppResult<ConversationAgent> {
        let Some(builder) = self.conversation_builders.get(config.provider.as_str()) else {
            return Err(AppError::internal(format!(
                "unsupported conversation provider: {}",
                config.provider.as_str()
            )));
        };
        builder(self, config, spec)
    }
}

#[async_trait]
impl TextModelGateway for RigModelFactory {
    async fn prompt_text(&self, request: TextPromptRequest) -> AppResult<String> {
        let Some(prompter) = self.text_prompters.get(request.model.provider.as_str()) else {
            return Err(AppError::internal(format!(
                "unsupported text prompt provider: {}",
                request.model.provider.as_str()
            )));
        };
        prompter(
            self,
            &request.model,
            request.preamble.as_str(),
            request.prompt,
        )?
        .run()
        .await
    }
}

impl RigModelFactory {
    fn openai_client(&self, config: &ResolvedAIModelConfig) -> AppResult<openai::Client> {
        openai::Client::builder()
            .api_key(&config.api_key)
            .base_url(&config.base_url)
            .build()
            .map_err(|e| AppError::upstream(e.to_string()))
    }

    fn deepseek_client(&self, config: &ResolvedAIModelConfig) -> AppResult<deepseek::Client> {
        deepseek::Client::builder()
            .api_key(&config.api_key)
            .base_url(&config.base_url)
            .build()
            .map_err(|e| AppError::upstream(e.to_string()))
    }
}

pub struct RagAgentContext {
    pub top_k: usize,
    pub index: MemoryRetriever,
}

pub struct AgentSpec {
    pub preamble: String,
    pub tool_choice: ToolChoice,
    pub memory: SessionConversationMemory,
    pub tools: Vec<Box<dyn ToolDyn>>,
    pub rag: Option<RagAgentContext>,
    pub guardrail: std::sync::Arc<GuardrailHook>,
    pub status_sink: Option<AgentStatusSink>,
}

pub enum ConversationAgent {
    OpenAiResponses(WarmmyAgent<OpenAiResponsesModel>),
    OpenAiCompletions(WarmmyAgent<OpenAiCompletionsModel>),
    DeepSeek(WarmmyAgent<DeepSeekModel>),
}

impl ConversationAgent {
    pub async fn prompt(self, prompt: Message, conversation_id: &str) -> AppResult<String> {
        match self {
            Self::OpenAiResponses(agent) => agent
                .prompt(prompt)
                .conversation(conversation_id)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
            Self::OpenAiCompletions(agent) => agent
                .prompt(prompt)
                .conversation(conversation_id)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
            Self::DeepSeek(agent) => agent
                .prompt(prompt)
                .conversation(conversation_id)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
        }
    }

    pub async fn stream(
        self,
        prompt: Message,
        conversation_id: String,
        wrap_ctx: StreamWrapCtx,
    ) -> AppResult<ConversationReplyStream> {
        match self {
            Self::OpenAiResponses(agent) => {
                let raw = agent
                    .stream_prompt(prompt)
                    .conversation(conversation_id)
                    .await;
                Ok(RigConversationRuntime::wrap_stream(raw, wrap_ctx))
            }
            Self::OpenAiCompletions(agent) => {
                let raw = agent
                    .stream_prompt(prompt)
                    .conversation(conversation_id)
                    .await;
                Ok(RigConversationRuntime::wrap_stream(raw, wrap_ctx))
            }
            Self::DeepSeek(agent) => {
                let raw = agent
                    .stream_prompt(prompt)
                    .conversation(conversation_id)
                    .await;
                Ok(RigConversationRuntime::wrap_stream(raw, wrap_ctx))
            }
        }
    }
}

enum TextPrompt {
    OpenAiCompletions {
        agent: Agent<OpenAiCompletionsModel>,
        prompt: String,
    },
    DeepSeek {
        agent: Agent<DeepSeekModel>,
        prompt: String,
    },
}

impl TextPrompt {
    async fn run(self) -> AppResult<String> {
        match self {
            Self::OpenAiCompletions { agent, prompt } => agent
                .prompt(prompt)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
            Self::DeepSeek { agent, prompt } => agent
                .prompt(prompt)
                .await
                .map_err(|e| AppError::upstream(e.to_string())),
        }
    }
}

fn configure_agent<M>(builder: AgentBuilder<M>, spec: AgentSpec) -> WarmmyAgent<M>
where
    M: CompletionModel + 'static,
{
    let hook = match spec.status_sink {
        Some(status_sink) => WarmmyPromptHook::with_status_sink(spec.guardrail, status_sink),
        None => WarmmyPromptHook::new(spec.guardrail),
    };

    let mut builder = builder
        .preamble(&spec.preamble)
        .tool_choice(spec.tool_choice)
        .default_max_turns(DEFAULT_MAX_TURNS)
        .memory(spec.memory);

    if let Some(rag) = spec.rag {
        builder = builder.dynamic_context(rag.top_k, rag.index);
    }

    builder.hook(hook).tools(spec.tools).build()
}

fn build_openai_responses_agent(
    factory: &RigModelFactory,
    config: &ResolvedAIModelConfig,
    spec: AgentSpec,
) -> AppResult<ConversationAgent> {
    let client = factory.openai_client(config)?;
    Ok(ConversationAgent::OpenAiResponses(configure_agent(
        client.agent(config.model.as_str()),
        spec,
    )))
}

fn build_openai_completions_agent(
    factory: &RigModelFactory,
    config: &ResolvedAIModelConfig,
    spec: AgentSpec,
) -> AppResult<ConversationAgent> {
    let client = factory.openai_client(config)?.completions_api();
    Ok(ConversationAgent::OpenAiCompletions(configure_agent(
        client.agent(config.model.as_str()),
        spec,
    )))
}

fn build_deepseek_agent(
    factory: &RigModelFactory,
    config: &ResolvedAIModelConfig,
    spec: AgentSpec,
) -> AppResult<ConversationAgent> {
    let client = factory.deepseek_client(config)?;
    Ok(ConversationAgent::DeepSeek(configure_agent(
        client.agent(config.model.as_str()),
        spec,
    )))
}

fn prompt_openai_completions(
    factory: &RigModelFactory,
    config: &ResolvedAIModelConfig,
    preamble: &str,
    prompt: String,
) -> AppResult<TextPrompt> {
    let agent = factory
        .openai_client(config)?
        .completions_api()
        .agent(config.model.as_str())
        .preamble(preamble)
        .build();
    Ok(TextPrompt::OpenAiCompletions { agent, prompt })
}

fn prompt_deepseek(
    factory: &RigModelFactory,
    config: &ResolvedAIModelConfig,
    preamble: &str,
    prompt: String,
) -> AppResult<TextPrompt> {
    let agent = factory
        .deepseek_client(config)?
        .agent(config.model.as_str())
        .preamble(preamble)
        .build();
    Ok(TextPrompt::DeepSeek { agent, prompt })
}
