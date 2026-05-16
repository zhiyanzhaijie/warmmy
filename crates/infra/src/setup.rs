use std::sync::Arc;

use adapters::prelude::{
    build_repos_by_url, BuiltinToolExecutor, HeuristicModelRouter, HitlCollaboration,
    LocalMemory, RigProviderConfig, RigRuntimeConfig, RuleBasedGuardrails, RuntimeLlmAdapter,
    SqliteNutritionRepo, TemplatePlanner,
};
use application::{
    app_error::{AppError, AppResult},
    common::agent::{
        CollaborationPort, GuardrailsPort, KnowledgeBasePort, ModelRoutingPort, PerceptionPort,
        PlanningPort, ReasoningPort, SessionMemoryPort, ToolExecutionPort,
    },
    meal::{MealCommandHandler, MealEventHandler, MealQueryHandler},
    user::UserProfileQueryHandler,
};

use crate::config::Config as AppConfig;

pub struct UserState {
    pub query: UserProfileQueryHandler,
}

pub struct MealState {
    pub query: MealQueryHandler,
    pub command: MealCommandHandler,
}

pub struct AdviceState {
    pub knowledge_base: Arc<dyn KnowledgeBasePort>,
}

pub struct NutritionState {
    pub repo: Arc<SqliteNutritionRepo>,
}
pub struct AgentCapabilitiesState {
    pub tool_execution: Arc<dyn ToolExecutionPort>,
    pub collaboration: Arc<dyn CollaborationPort>,
    pub model_routing: Arc<dyn ModelRoutingPort>,
}

pub struct AppContainer {
    pub config: AppConfig,
    pub user: UserState,
    pub meal: MealState,
    pub advice: AdviceState,
    pub nutrition: NutritionState,
    pub agent_capabilities: AgentCapabilitiesState,
}

/// The server/runtime state injected into HTTP handlers.
///
/// For now we reuse `AppContainer` directly as the state type.
pub type AppState = AppContainer;

pub async fn init_app_container() -> AppResult<AppContainer> {
    let config = AppConfig::load().map_err(AppError::internal)?;
    let repos = build_repos_by_url(&config.database.url).await?;

    let reasoning_route = config
        .llm
        .resolve_route(&config.llm.routing.reasoning)
        .map_err(AppError::internal)?;
    let perception_route = config
        .llm
        .resolve_route(&config.llm.routing.perception)
        .map_err(AppError::internal)?;

    let reasoning: Arc<dyn ReasoningPort> = Arc::new(RuntimeLlmAdapter::new(
        RigRuntimeConfig {
            model: reasoning_route.model,
            provider: RigProviderConfig {
                name: reasoning_route.provider,
                base_url: reasoning_route.base_url,
                api_key: reasoning_route.api_key,
            },
        },
        config.llm.enable_image_parsing,
    ));
    let perception: Arc<dyn PerceptionPort> = Arc::new(RuntimeLlmAdapter::new(
        RigRuntimeConfig {
            model: perception_route.model,
            provider: RigProviderConfig {
                name: perception_route.provider,
                base_url: perception_route.base_url,
                api_key: perception_route.api_key,
            },
        },
        config.llm.enable_image_parsing,
    ));
    let memory = Arc::new(LocalMemory::default());
    let memory_for_command: Arc<dyn SessionMemoryPort> = memory.clone();
    let memory_for_query: Arc<dyn SessionMemoryPort> = memory;
    let planning: Arc<dyn PlanningPort> = Arc::new(TemplatePlanner::default());
    let guardrails: Arc<dyn GuardrailsPort> = Arc::new(RuleBasedGuardrails::default());
    let tool_execution: Arc<dyn ToolExecutionPort> = Arc::new(BuiltinToolExecutor);
    let collaboration: Arc<dyn CollaborationPort> = Arc::new(HitlCollaboration::default());
    let model_routing: Arc<dyn ModelRoutingPort> = Arc::new(HeuristicModelRouter::default());

    let user = UserState {
        query: UserProfileQueryHandler::new(repos.user_repo.clone()),
    };

    let meal = MealState {
        query: MealQueryHandler::new(memory_for_query),
        command: MealCommandHandler::new(
            reasoning,
            perception,
            planning,
            guardrails,
            memory_for_command,
            repos.user_repo.clone(),
            repos.meal_repo.clone(),
            repos.advice_repo.clone(),
        )
        .with_event_handler(MealEventHandler::new()),
    };

    let advice = AdviceState {
        knowledge_base: repos.advice_repo.clone(),
    };

    let nutrition = NutritionState {
        repo: repos.nutrition_repo.clone(),
    };
    let agent_capabilities = AgentCapabilitiesState {
        tool_execution,
        collaboration,
        model_routing,
    };

    Ok(AppContainer {
        config,
        user,
        meal,
        advice,
        nutrition,
        agent_capabilities,
    })
}
