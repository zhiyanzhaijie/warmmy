use async_trait::async_trait;
use application::common::agent::ToolExecutionPort;

#[derive(Debug, Clone, Default)]
pub struct BuiltinToolExecutor;

#[async_trait]
impl ToolExecutionPort for BuiltinToolExecutor {
    async fn execute_tool(&self, tool_name: &str, payload: &str) -> Result<String, String> {
        match tool_name {
            "text.normalize" => Ok(payload.trim().split_whitespace().collect::<Vec<_>>().join(" ")),
            "text.word_count" => Ok(payload.split_whitespace().count().to_string()),
            "json.wrap" => Ok(format!("{{\"value\":\"{}\"}}", payload.replace('"', "\\\""))),
            _ => Err(format!("unsupported tool: {tool_name}")),
        }
    }
}
