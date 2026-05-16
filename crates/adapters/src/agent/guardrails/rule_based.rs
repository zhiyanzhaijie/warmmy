use async_trait::async_trait;
use application::common::agent::{GuardrailDecision, GuardrailsPort};

#[derive(Debug, Clone)]
pub struct RuleBasedGuardrails {
    max_input_chars: usize,
    blocked_input_patterns: Vec<&'static str>,
    blocked_effect_patterns: Vec<&'static str>,
    blocked_output_patterns: Vec<&'static str>,
}

impl Default for RuleBasedGuardrails {
    fn default() -> Self {
        Self {
            max_input_chars: 8_000,
            blocked_input_patterns: vec![
                "ignore previous instructions",
                "system prompt",
                "jailbreak",
                "bypass safety",
                "developer message",
            ],
            blocked_effect_patterns: vec![
                "drop table",
                "delete from",
                "rm -rf",
                "overwrite",
                "exfiltrate",
            ],
            blocked_output_patterns: vec![
                "sk-",
                "-----BEGIN",
                "@",
                "password",
                "api key",
            ],
        }
    }
}

impl RuleBasedGuardrails {
    fn contains_blocked_pattern(value: &str, patterns: &[&'static str]) -> Option<&'static str> {
        let lower = value.to_lowercase();
        patterns
            .iter()
            .find(|pattern| lower.contains(**pattern))
            .copied()
    }

    fn evaluate(value: &str, patterns: &[&'static str], empty_err: &str) -> GuardrailDecision {
        if value.trim().is_empty() {
            return GuardrailDecision::Reject(empty_err.to_string());
        }
        if let Some(pattern) = Self::contains_blocked_pattern(value, patterns) {
            return GuardrailDecision::Reject(format!("contains blocked pattern: {pattern}"));
        }
        GuardrailDecision::Allow
    }
}

#[async_trait]
impl GuardrailsPort for RuleBasedGuardrails {
    async fn check_input(&self, input: &str) -> Result<GuardrailDecision, String> {
        if input.chars().count() > self.max_input_chars {
            return Ok(GuardrailDecision::Reject(format!(
                "input is too long: exceeds {} characters",
                self.max_input_chars
            )));
        }
        Ok(Self::evaluate(
            input,
            &self.blocked_input_patterns,
            "empty input is not allowed",
        ))
    }

    async fn check_effect(&self, effect: &str) -> Result<GuardrailDecision, String> {
        Ok(Self::evaluate(
            effect,
            &self.blocked_effect_patterns,
            "empty effect is not allowed",
        ))
    }

    async fn check_output(&self, output: &str) -> Result<GuardrailDecision, String> {
        Ok(Self::evaluate(
            output,
            &self.blocked_output_patterns,
            "empty output is not allowed",
        ))
    }
}
