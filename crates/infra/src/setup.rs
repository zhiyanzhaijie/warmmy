use std::sync::Arc;

use adapters::agent::builder::{AgentConfig, ConversationAgent};
use adapters::agent::retrieval::RetrievalConfig;
use adapters::prelude::{build_repos_by_url, SqliteNutritionRepo};
use app::app_error::{AppError, AppResult};
use app::common::agent::KnowledgeBasePort;
use app::conversation::{ConversationCommandHandler, ConversationQueryHandler};
use app::meal::{MealCommandHandler, MealEventHandler, MealQueryHandler};
use app::user::UserProfileQueryHandler;

use crate::config::Config as AppConfig;

pub struct UserState {
    pub query: UserProfileQueryHandler,
}

pub struct MealState {
    pub command: MealCommandHandler,
    pub query: MealQueryHandler,
}

pub struct AdviceState {
    pub knowledge_base: Arc<dyn KnowledgeBasePort>,
}

pub struct ConversationState {
    pub command: ConversationCommandHandler,
    pub query: ConversationQueryHandler,
}

pub struct NutritionState {
    pub repo: Arc<SqliteNutritionRepo>,
}

pub struct AppContainer {
    pub config: AppConfig,
    pub user: UserState,
    pub meal: MealState,
    pub advice: AdviceState,
    pub conversation: ConversationState,
    pub nutrition: NutritionState,
}

pub type AppState = AppContainer;

pub async fn init_app_container() -> AppResult<AppContainer> {
    let config = AppConfig::load().map_err(AppError::internal)?;
    let repos = build_repos_by_url(&config.database.url).await?;

    let route = config
        .llm
        .resolve_route(&config.llm.routing.reasoning)
        .map_err(AppError::internal)?;
    let embedding_route = config
        .llm
        .resolve_route(&config.llm.routing.embedding)
        .map_err(AppError::internal)?;

    let agent_config = AgentConfig {
        provider: route.provider,
        base_url: route.base_url,
        api_key: route.api_key,
        model: route.model,
        retrieval: RetrievalConfig {
            lancedb_path: config.retrieval.lancedb_path.clone(),
            embedding_provider: embedding_route.provider,
            embedding_base_url: embedding_route.base_url,
            embedding_api_key: embedding_route.api_key,
            embedding_model: embedding_route.model,
            embedding_ndims: config.retrieval.embedding_ndims,
            top_k: config.retrieval.top_k,
        },
    };

    let user = UserState {
        query: UserProfileQueryHandler::new(repos.user_repo.clone()),
    };

    let meal_command = Arc::new(
        MealCommandHandler::new(repos.user_repo.clone(), repos.meal_repo.clone())
            .with_event_handler(MealEventHandler::new()),
    );

    let meal = MealState {
        command: meal_command.as_ref().clone(),
        query: MealQueryHandler::new(repos.meal_repo.clone()),
    };

    let advice = AdviceState {
        knowledge_base: repos.advice_repo.clone(),
    };

    let conversation = ConversationState {
        command: ConversationCommandHandler::new(Arc::new(ConversationAgent::new(
            agent_config,
            meal_command,
            repos.chat_repo.clone(),
        ))),
        query: ConversationQueryHandler::new(repos.chat_repo.clone()),
    };

    let nutrition = NutritionState {
        repo: repos.nutrition_repo.clone(),
    };

    Ok(AppContainer {
        config,
        user,
        meal,
        advice,
        conversation,
        nutrition,
    })
}
