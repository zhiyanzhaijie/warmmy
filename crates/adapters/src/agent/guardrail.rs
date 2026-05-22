pub struct GuardrailHook;

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
