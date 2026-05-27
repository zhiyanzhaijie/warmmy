use std::sync::Arc;

use app::app_error::AppResult;
use app::conversation::{
    AgentInteractionContinuation, ChatMessageRepositoryPort, ContinueInteractionCommand,
    ConversationAgentPort, ConversationReplyStream, SendUserMessageCommand, SendUserMessageResult,
};
use app::meal::MealCommandHandler;
use app::user::UserDietaryContextQueryHandler;
use async_trait::async_trait;

use crate::agent::config::AgentConfig;
use crate::agent::runtime::rig::RigConversationRuntime;

pub struct ConversationAgentService {
    runtime: RigConversationRuntime,
}

impl ConversationAgentService {
    pub fn new(
        config: AgentConfig,
        meal_command: Arc<MealCommandHandler>,
        repo: Arc<dyn ChatMessageRepositoryPort>,
        user_contexts: UserDietaryContextQueryHandler,
    ) -> Self {
        let runtime = RigConversationRuntime::new(config, meal_command, repo, user_contexts);
        Self { runtime }
    }
}

#[async_trait]
impl ConversationAgentPort for ConversationAgentService {
    async fn send_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<SendUserMessageResult> {
        self.runtime
            .complete(&command.user_id, &command.session_id, &command.content)
            .await
    }

    async fn stream_user_message(
        &self,
        command: SendUserMessageCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.runtime
            .stream(&command.user_id, &command.session_id, command.content)
            .await
    }

    async fn continue_interaction(
        &self,
        command: ContinueInteractionCommand,
    ) -> AppResult<ConversationReplyStream> {
        self.runtime
            .stream(
                &command.user_id,
                &command.session_id,
                interaction_continuation_prompt(command.interaction),
            )
            .await
    }
}

const INTERNAL_CONTINUATION_MARKER: &str = "[warmmy:internal-continuation]";

fn interaction_continuation_prompt(interaction: AgentInteractionContinuation) -> String {
    match interaction {
        AgentInteractionContinuation::ConfirmMealLog { pending_id } => format!(
            "{INTERNAL_CONTINUATION_MARKER} 用户已在界面确认一条待确认用餐记录。该草稿已经按用户最终编辑内容重新估算并保存。请立即调用 confirm_meal_log 工具正式保存，不要重新询问。pending_id: {pending_id}。工具成功后，用自然中文告诉用户已经记录，并基于记录给出简短说明。"
        ),
        AgentInteractionContinuation::RejectMealLog { pending_id } => format!(
            "{INTERNAL_CONTINUATION_MARKER} 用户已在界面取消一条待确认用餐记录。请立即调用 reject_meal_log 工具删除草稿，不要重新询问。pending_id: {pending_id}。工具成功后，用自然中文告诉用户没有写入正式记录。"
        ),
        AgentInteractionContinuation::SummarizeMealDay {
            finalized_at,
            meals_json,
        } => format!(
            "{INTERNAL_CONTINUATION_MARKER} 用户已主动敲定今天的餐食记录，finalized_at: {finalized_at}。今天不再接受新的 meal log。请根据下面的正式 meal records、Current Facts 中的健康期望和饮食偏好，输出一份亲密、具体、简洁的今日饮食回顾。要求包含：1. 今日总体评价；2. 营养亮点；3. 可能的不足或明天调整建议；4. 一句鼓励。不要调用 meal log 工具，不要要求用户再确认。\n\n正式 meal records JSON:\n{meals_json}"
        ),
    }
}
