use std::sync::Arc;

use app::agents::{
    prompts::memory_extractor::MEMORY_EXTRACTOR_PREAMBLE, IndexPolicy, LongTermMemoryForm,
    MemoryCandidate, MemoryClaim, MemoryExtractor, MemoryObservation, MemoryScope, MemorySource,
    MemoryStatus, TextModelGateway, TextPromptRequest,
};
use app::app_error::{AppError, AppResult};
use app::user::ResolvedAIModelConfig;
use async_trait::async_trait;
use serde::Deserialize;

use crate::agent::model::RigModelFactory;

#[derive(Clone)]
pub struct ModelMemoryExtractor {
    model: ResolvedAIModelConfig,
    text_model: Arc<dyn TextModelGateway>,
}

impl ModelMemoryExtractor {
    pub fn new(model: ResolvedAIModelConfig) -> Self {
        Self {
            model,
            text_model: Arc::new(RigModelFactory::new()),
        }
    }

    pub fn with_model_gateway(
        model: ResolvedAIModelConfig,
        text_model: Arc<dyn TextModelGateway>,
    ) -> Self {
        Self { model, text_model }
    }

    async fn prompt_model(&self, observation: &MemoryObservation) -> AppResult<String> {
        let prompt = format!(
            "reference_date：{}\n用户本轮输入：\n{}",
            observation.reference_date, observation.user_input
        );

        self.text_model
            .prompt_text(TextPromptRequest {
                model: self.model.clone(),
                preamble: MEMORY_EXTRACTOR_PREAMBLE.to_string(),
                prompt,
            })
            .await
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
