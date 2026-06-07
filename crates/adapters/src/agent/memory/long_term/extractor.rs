use app::app_error::{AppError, AppResult};
use app::user::ResolvedAIModelConfig;
use async_trait::async_trait;
use domain::UserId;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{deepseek, openai};
use serde::Deserialize;

use crate::agent::memory::types::{
    IndexPolicy, LongTermMemoryForm, MemoryCandidate, MemoryClaim, MemoryScope, MemorySource,
    MemoryStatus,
};

const MEMORY_EXTRACTOR_PREAMBLE: &str = r#"你是 warmmy 的长期记忆抽取器。

你的任务是在一轮对话结束后，只从用户本轮输入中判断是否有值得未来使用的长期记忆候选。

记忆只分为：
- semantic: 用户相对稳定的事实、偏好、关系、身份、健康背景、生活上下文
- episodic: 某天或某段时间发生的经历、事件、状态变化
- procedural: 用户希望 warmmy 以后如何互动、称呼、解释、提醒、配合

长期记忆的 scope 只允许：
- user_global: 跨日期、跨会话长期有效的事实或偏好
- day: 和明确日期绑定的经历或上下文，必须提供 date
- temporary: 会跨会话短暂影响照顾方式的状态，必须提供 expires_at
不要输出 session scope。当前会话内上下文由 Rig ConversationMemory 管理，不进入长期记忆库。

输入会提供 reference_date，它来自当前会话的 session_id。
- 用 reference_date 理解所有语言中的相对日期和相对时间，例如今天、今早、近期、recently、this morning、for now
- 如果用户表达当天发生的经历，day.date 必须使用 reference_date
- 如果用户表达临时状态但没有明确截止时间，temporary.expires_at 应使用 reference_date 之后 14 天的 23:59:59+08:00
- 不要使用模型自己的当前日期猜测 date 或 expires_at

不要把业务事实当成长期语义记忆：
- 某天某餐吃了什么，只有用户确认后才进入 meal_log
- 未确认用餐只属于 pending_meal_log
- 不要把单次餐食流水写成 semantic memory
- 如果一句话同时包含餐食和其他长期/跨会话上下文，只排除具体餐食流水，不要丢弃其他记忆事实

可以记录：
- 用户希望如何称呼自己，用户给 warmmy 的昵称，双方稳定称谓
- 用户近期重要经历、压力源、情绪背景、计划目标
- 用户饮食偏好、口味倾向、忌口表达、过敏风险
- 用户健康期望、饮食控制策略、需要持续照顾的健康上下文
- 用户做饭场景、厨具、采购习惯、作息、常见就餐对象
- 用户的重要关系，以及关系中的饮食/健康上下文
- 某天去了某个重要关系或地点，以及当时发生的非餐食事件
- 近期会影响饮食建议的临时身体状态或伤病，例如口腔受伤、胃不舒服、医生临时要求忌口

不要记录：
- 寒暄、普通闲聊、无事实价值的临时情绪
- 用户没有确认的具体 meal log
- 密码、API key、支付凭证、验证码、身份证号等秘密或高风险凭证
- 助手回复、历史记忆复述、RAG 检索结果
- 用户只是询问“你是否记得”而没有提供新事实

严格只返回 JSON，不要输出 Markdown，不要解释。
格式：
{
  "memories": [
    {
      "content": "一条自包含的中文长期记忆候选",
      "form": "semantic|episodic|procedural",
      "scope": "day|user_global|temporary",
      "date": "YYYY-MM-DD 或空字符串",
      "expires_at": "RFC3339 或空字符串",
      "index": true,
      "claim": {"subject": "user", "predicate": "home_area"} 或 null,
      "importance": 0.0
    }
  ]
}
示例：
用户说“今天去姨妈家吃了五花肉，咬伤舌头，近期不能吃辛辣。”
应提取“用户今天去了姨妈家。”作为 episodic/day；提取“用户近期咬伤舌头，暂时不能吃辛辣。”作为 semantic/temporary 或 episodic/temporary；不要提取“吃了五花肉”作为长期记忆。
如果没有值得记录的内容，返回：{"memories":[]}"#;

#[derive(Clone, Debug)]
pub struct MemoryObservation {
    pub user_id: UserId,
    pub session_id: String,
    pub reference_date: String,
    pub user_input: String,
    pub assistant_output: String,
}

#[async_trait]
pub trait MemoryExtractor: Send + Sync {
    async fn extract(&self, observation: MemoryObservation) -> AppResult<Vec<MemoryCandidate>>;
}

#[derive(Clone, Default)]
pub struct NoopMemoryExtractor;

#[async_trait]
impl MemoryExtractor for NoopMemoryExtractor {
    async fn extract(&self, _observation: MemoryObservation) -> AppResult<Vec<MemoryCandidate>> {
        Ok(Vec::new())
    }
}

#[derive(Clone)]
pub struct ModelMemoryExtractor {
    model: ResolvedAIModelConfig,
}

impl ModelMemoryExtractor {
    pub fn new(model: ResolvedAIModelConfig) -> Self {
        Self { model }
    }

    async fn prompt_model(&self, observation: &MemoryObservation) -> AppResult<String> {
        let prompt = format!(
            "reference_date：{}\n用户本轮输入：\n{}",
            observation.reference_date, observation.user_input
        );

        match self.model.provider.as_str() {
            "openai" | "openai_compatible" | "siliconflow" | "dashscope" => {
                let client = openai::Client::builder()
                    .api_key(&self.model.api_key)
                    .base_url(&self.model.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?
                    .completions_api();
                client
                    .agent(self.model.model.as_str())
                    .preamble(MEMORY_EXTRACTOR_PREAMBLE)
                    .build()
                    .prompt(prompt)
                    .await
                    .map_err(|e| AppError::upstream(e.to_string()))
            }
            "deepseek" => {
                let client = deepseek::Client::builder()
                    .api_key(&self.model.api_key)
                    .base_url(&self.model.base_url)
                    .build()
                    .map_err(|e| AppError::upstream(e.to_string()))?;
                client
                    .agent(self.model.model.as_str())
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

#[async_trait]
impl MemoryExtractor for ModelMemoryExtractor {
    async fn extract(&self, observation: MemoryObservation) -> AppResult<Vec<MemoryCandidate>> {
        let raw = self.prompt_model(&observation).await?;
        let extracted = parse_extractor_output(&raw)?;
        tracing::info!(
            session.id = observation.session_id.as_str(),
            extracted.count = extracted.memories.len(),
            "memory extractor returned candidates"
        );

        let mut candidates = Vec::new();
        for memory in extracted.memories {
            if let Some(candidate) = candidate_from_extracted(&observation, memory)? {
                tracing::info!(
                    session.id = observation.session_id.as_str(),
                    memory.form = ?candidate.form,
                    memory.scope = ?candidate.scope,
                    memory.index_policy = ?candidate.index_policy,
                    memory.confidence = candidate.confidence,
                    "memory candidate accepted"
                );
                candidates.push(candidate);
            }
        }
        Ok(candidates)
    }
}

#[derive(Debug, Deserialize)]
struct ExtractedMemories {
    memories: Vec<ExtractedMemory>,
}

#[derive(Debug, Deserialize)]
struct ExtractedMemory {
    content: String,
    form: String,
    scope: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    expires_at: String,
    #[serde(default)]
    index: bool,
    #[serde(default)]
    claim: Option<ExtractedClaim>,
    #[serde(default)]
    importance: f32,
}

#[derive(Debug, Deserialize)]
struct ExtractedClaim {
    subject: String,
    predicate: String,
}

fn candidate_from_extracted(
    observation: &MemoryObservation,
    memory: ExtractedMemory,
) -> AppResult<Option<MemoryCandidate>> {
    let content = memory.content.trim();
    if content.is_empty() {
        tracing::info!("memory candidate dropped: empty content");
        return Ok(None);
    }

    let Some(scope) = scope_from_extracted(&memory)? else {
        tracing::info!(
            memory.form = memory.form.as_str(),
            memory.scope = memory.scope.as_str(),
            has_date = !memory.date.trim().is_empty(),
            has_expires_at = !memory.expires_at.trim().is_empty(),
            "memory candidate dropped: incomplete scope"
        );
        return Ok(None);
    };

    Ok(Some(MemoryCandidate {
        user_id: observation.user_id.clone(),
        form: form_from_str(&memory.form)?,
        scope,
        source: MemorySource::UserUtterance {
            session_id: observation.session_id.clone(),
        },
        status: MemoryStatus::Candidate,
        index_policy: if memory.index {
            IndexPolicy::Vector
        } else {
            IndexPolicy::None
        },
        content: content.to_string(),
        search_text: search_text(content, memory.importance),
        confidence: memory.importance.clamp(0.0, 1.0),
        claim: memory.claim.and_then(claim_from_extracted),
    }))
}

fn claim_from_extracted(claim: ExtractedClaim) -> Option<MemoryClaim> {
    let subject = claim.subject.trim();
    let predicate = claim.predicate.trim();
    if subject.is_empty() || predicate.is_empty() {
        return None;
    }
    Some(MemoryClaim {
        subject: subject.to_string(),
        predicate: predicate.to_string(),
    })
}

fn form_from_str(value: &str) -> AppResult<LongTermMemoryForm> {
    match value.trim() {
        "semantic" => Ok(LongTermMemoryForm::Semantic),
        "episodic" => Ok(LongTermMemoryForm::Episodic),
        "procedural" => Ok(LongTermMemoryForm::Procedural),
        other => Err(AppError::validation(format!(
            "unknown extracted memory form: {other}"
        ))),
    }
}

fn scope_from_extracted(memory: &ExtractedMemory) -> AppResult<Option<MemoryScope>> {
    match memory.scope.trim() {
        "day" => {
            let date = memory.date.trim();
            if date.is_empty() {
                return Ok(None);
            }
            Ok(Some(MemoryScope::Day {
                date: date.to_string(),
            }))
        }
        "user_global" => Ok(Some(MemoryScope::UserGlobal)),
        "temporary" => {
            let expires_at = memory.expires_at.trim();
            if expires_at.is_empty() {
                return Ok(None);
            }
            Ok(Some(MemoryScope::Temporary {
                expires_at: expires_at.to_string(),
            }))
        }
        other => Err(AppError::validation(format!(
            "unknown extracted memory scope: {other}"
        ))),
    }
}

fn search_text(content: &str, importance: f32) -> String {
    if importance > 0.0 {
        format!("{content}\nimportance: {importance:.2}")
    } else {
        content.to_string()
    }
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
