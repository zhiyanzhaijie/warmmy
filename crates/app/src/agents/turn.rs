use std::sync::Arc;

use async_trait::async_trait;

use crate::app_error::AppResult;
use crate::user::ResolvedAIModelConfig;

use super::routing::{
    AgentRoute, AgentToolId, RouteDecision, RouteInput, RoutePlanner, RouteRegistry,
};

const WARMMY_AGENT_PREAMBLE: &str = r#"你是 warmmy，一个温暖、专业的对话型饮食助理。

## 自我认知
- 你可以自然聊天，回答营养健康相关问题
- 你会优先使用 Current Context 理解用户偏好、忌口、过敏原和健康期望
- 当用户的问题同时包含画像询问、饮食记录、分析或建议时，在同一次回答中完整处理，不要只回答其中一部分

## 语言
- 始终使用中文回复"#;

#[derive(Clone)]
pub struct AgentTurnInput {
    pub text: String,
    pub has_images: bool,
    pub is_internal_input: bool,
    pub model: ResolvedAIModelConfig,
    pub current_context: String,
    pub chat_enabled: bool,
    pub semantic_memory_enabled: bool,
    pub memory_observation_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct AgentTurnPlan {
    pub route_decision: RouteDecision,
    pub preamble: String,
    pub tool_ids: Vec<AgentToolId>,
    pub lifecycle: AgentTurnLifecycle,
}

#[derive(Clone, Debug)]
pub struct AgentTurnLifecycle {
    pub persist_user_visible_message: bool,
    pub persist_user_image_message: bool,
    pub persist_assistant_visible_message: bool,
    pub memory_observation_enabled: bool,
    pub update_conversation_summary: bool,
}

impl AgentTurnPlan {
    pub fn route(&self) -> AgentRoute {
        self.route_decision.route
    }
}

#[async_trait]
pub trait AgentTurnPlanner: Send + Sync {
    async fn plan_turn(&self, input: AgentTurnInput) -> AppResult<AgentTurnPlan>;
}

#[derive(Clone)]
pub struct DefaultAgentTurnPlanner {
    route_planner: Arc<dyn RoutePlanner>,
    route_registry: RouteRegistry,
}

impl DefaultAgentTurnPlanner {
    pub fn new(route_planner: Arc<dyn RoutePlanner>, route_registry: RouteRegistry) -> Self {
        Self {
            route_planner,
            route_registry,
        }
    }
}

#[async_trait]
impl AgentTurnPlanner for DefaultAgentTurnPlanner {
    async fn plan_turn(&self, input: AgentTurnInput) -> AppResult<AgentTurnPlan> {
        let route_decision = self
            .route_planner
            .plan(RouteInput {
                text: input.text.clone(),
                has_images: input.has_images,
                model: input.model,
            })
            .await?;
        let tool_ids = self.route_registry.tool_ids(route_decision.route).to_vec();

        let has_visible_text = !input.text.trim().is_empty();
        let lifecycle = AgentTurnLifecycle {
            persist_user_visible_message: has_visible_text
                && !input.has_images
                && !input.is_internal_input,
            persist_user_image_message: input.has_images,
            persist_assistant_visible_message: true,
            memory_observation_enabled: input.memory_observation_enabled
                && has_visible_text
                && !input.is_internal_input,
            update_conversation_summary: true,
        };

        Ok(AgentTurnPlan {
            route_decision,
            preamble: build_warmmy_agent_preamble(
                &input.current_context,
                input.chat_enabled,
                input.semantic_memory_enabled,
                input.has_images,
            ),
            tool_ids,
            lifecycle,
        })
    }
}

pub fn build_warmmy_agent_preamble(
    context: &str,
    chat_enabled: bool,
    semantic_memory_enabled: bool,
    vision_enabled: bool,
) -> String {
    let chat_status = if chat_enabled {
        "聊天功能已启用。"
    } else {
        "聊天功能未启用。"
    };
    let semantic_status = if semantic_memory_enabled {
        "长期语义记忆/RAG 已启用。"
    } else {
        "长期语义记忆/RAG 未启用：用户尚未配置 embedding 模型或 API key。只能使用当前会话记忆和 Current Context。"
    };
    let vision_status = if vision_enabled {
        "识图功能本轮已启用。"
    } else {
        "识图功能本轮未启用。"
    };
    format!(
        "{WARMMY_AGENT_PREAMBLE}\n\n## Capability Status\n- {chat_status}\n- {semantic_status}\n- {vision_status}\n\n## Current Context\n{context}"
    )
}
