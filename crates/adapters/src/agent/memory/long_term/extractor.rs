use app::app_error::{AppError, AppResult};
use domain::UserId;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{deepseek, openai};
use serde::Deserialize;

use crate::agent::config::AgentModelConfig;
use crate::agent::memory::long_term::rag::{embed_rag_document, RagConfig, RagDocument};

const MEMORY_EXTRACTOR_PREAMBLE: &str = r#"你是 warmmy 的长期语义记忆整理器。

你的任务是在一轮对话结束后，只从用户本轮输入中判断是否有值得未来检索使用的长期语义记忆。

记忆的核心标准是“用户自我公开的主体特殊性”：凡是能帮助 warmmy 在未来更理解这个用户是谁、如何称呼他、他正在经历什么、他在意谁、他的健康和饮食处境如何的信息，都应该考虑记录。
不仅记录饮食和健康，也记录用户主动公开的稳定身份线索、称谓偏好、近期经历、关系网络、生活场景、情绪背景和计划目标。
不要记录寒暄、纯临时情绪、无事实价值的普通对话、已经明显过期且未来不可用的内容。
不要从助手回复、历史记忆复述、RAG 检索结果中提取记忆。
如果用户只是在询问你是否记得某个信息，而没有提供新的事实，返回空 memories。
不要记录密码、API key、支付凭证、验证码、身份证号等秘密或高风险凭证。
涉及第三方的敏感信息时，只在用户主动公开且对未来陪伴、饮食、健康或关系理解明显有用时，概括记录必要上下文，不要扩写、臆测或添加未说出的事实。

适合记录：
- 用户希望 warmmy 如何称呼自己，用户给 warmmy 的昵称，双方稳定的互相称谓
- 用户近期发生的重要事情、正在经历的变化、压力源、情绪背景、需要被持续照顾的经验
- 用户饮食偏好、口味倾向、忌口表达、过敏风险
- 用户健康期望、近期计划、饮食控制策略
- 用户做饭场景、厨具、采购习惯、作息、常见就餐对象
- 用户的重要关系：伴侣、家人、朋友、同事、室友，以及最近和谁吃饭、谁来家里、和谁发生矛盾
- 关系中的饮食/健康上下文：某人忌口、过敏、偏好、需要照顾的饮食特殊情况
- 用户反复出现或可用于未来建议的餐食模式

不要把权威事实改写成命令；只输出可被 RAG 检索并直接给模型阅读的自然语言记忆。

严格只返回 JSON，不要输出 Markdown，不要解释。
格式：
{
  "memories": [
    {
      "content": "一条自包含的中文长期记忆",
      "kind": "identity_context|relationship_context|experience|preference|habit|health_expectation|household_context|social_context|meal_pattern|other",
      "importance": 0.0
    }
  ]
}
如果没有值得记录的内容，返回：{"memories":[]}"#;

#[derive(Clone)]
pub struct LongTermMemoryExtractor {
    chat: AgentModelConfig,
    rag: RagConfig,
}

impl LongTermMemoryExtractor {
    pub fn new(chat: AgentModelConfig, rag: RagConfig) -> Self {
        Self { chat, rag }
    }

    pub async fn extract_and_embed(&self, user_id: &UserId, user_input: &str) -> AppResult<()> {
        let raw = self.extract(user_input).await?;
        let memories = parse_extractor_output(&raw)?;

        if memories.memories.is_empty() {
            tracing::info!(
                user.id = user_id.as_str(),
                "memory extractor found no memories"
            );
            return Ok(());
        }

        let now = chrono::Utc::now();
        let now_nanos = now
            .timestamp_nanos_opt()
            .unwrap_or_else(|| now.timestamp_micros());

        for (index, memory) in memories.memories.into_iter().enumerate() {
            let content = memory.content.trim();
            if content.is_empty() {
                continue;
            }

            let document = RagDocument {
                id: format!("memory:{}:{}:{}", user_id.as_str(), now_nanos, index),
                content: format!("用户{}长期语义记忆：{}", user_id.as_str(), content),
                source: format!(
                    "conversation_memory:{}:{:.2}",
                    memory.kind.trim(),
                    memory.importance
                ),
            };

            embed_rag_document(&self.rag, document).await?;
        }

        Ok(())
    }

    async fn extract(&self, user_input: &str) -> AppResult<String> {
        let prompt = format!("用户本轮输入：\n{user_input}");

        match self.chat.provider.as_str() {
            "openai" => {
                let client = openai::Client::builder()
                    .api_key(&self.chat.api_key)
                    .base_url(&self.chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                client
                    .agent(self.chat.model.as_str())
                    .preamble(MEMORY_EXTRACTOR_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.chat.api_key)
                    .base_url(&self.chat.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                client
                    .agent(self.chat.model.as_str())
                    .preamble(MEMORY_EXTRACTOR_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))
            }
            provider => Err(AppError::internal(format!(
                "unsupported memory extractor provider: {provider}"
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExtractedMemories {
    memories: Vec<ExtractedMemory>,
}

#[derive(Debug, Deserialize)]
struct ExtractedMemory {
    content: String,
    kind: String,
    #[serde(default)]
    importance: f32,
}

fn parse_extractor_output(raw: &str) -> AppResult<ExtractedMemories> {
    let cleaned = raw
        .trim()
        .strip_prefix("```json")
        .unwrap_or(raw.trim())
        .trim()
        .strip_prefix("```")
        .unwrap_or_else(|| {
            raw.trim()
                .strip_prefix("```json")
                .unwrap_or(raw.trim())
                .trim()
        })
        .trim()
        .strip_suffix("```")
        .unwrap_or_else(|| raw.trim())
        .trim();

    serde_json::from_str(cleaned).map_err(|e| {
        AppError::upstream(format!(
            "memory extractor returned invalid json: {e}; raw={raw}"
        ))
    })
}
