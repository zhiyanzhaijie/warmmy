use std::sync::Arc;

use adapters::prelude::{
    build_repos_by_url, OpenAiCompatibleLlm, PsqlNutritionRepo, QdrantMemory,
};
use application::{
    app_error::AppResult,
    advice::KnowledgeBasePort,
    meal::{LlmPort, MealCommandHandler, MealEventHandler, MealQueryHandler, SessionMemoryPort},
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
    pub repo: Arc<PsqlNutritionRepo>,
}

pub struct AppContainer {
    pub config: AppConfig,
    pub user: UserState,
    pub meal: MealState,
    pub advice: AdviceState,
    pub nutrition: NutritionState,
}

/// The server/runtime state injected into HTTP handlers.
///
/// For now we reuse `AppContainer` directly as the state type.
pub type AppState = AppContainer;

pub async fn init_app_container() -> AppResult<AppContainer> {
    let config = AppConfig::load();
    let repos = build_repos_by_url(&config.database.url).await?;

    let llm: Arc<dyn LlmPort> = Arc::new(OpenAiCompatibleLlm::new(config.llm.model.clone()));
    let memory = Arc::new(QdrantMemory::default());
    let memory_for_command: Arc<dyn SessionMemoryPort> = memory.clone();
    let memory_for_query: Arc<dyn SessionMemoryPort> = memory;

    let user = UserState {
        query: UserProfileQueryHandler::new(repos.user_repo.clone()),
    };

    let meal = MealState {
        query: MealQueryHandler::new(memory_for_query),
        command: MealCommandHandler::new(
            llm,
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

    Ok(AppContainer {
        config,
        user,
        meal,
        advice,
        nutrition,
    })
}
