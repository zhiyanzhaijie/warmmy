use adapters::agent::config::AgentConfig;
use adapters::agent::memory::long_term::rag::RagConfig;
use adapters::agent::service::ConversationAgentService;
use adapters::prelude::build_repos_by_url;
use app::app_error::{AppError, AppResult};
use app::conversation::{ConversationCommandHandler, ConversationQueryHandler};
use app::meal::FoodNutritionReferenceRepositoryPort;
use app::meal::{MealCommandHandler, MealEventHandler, MealQueryHandler};
use app::user::{
    DiningCompanionCommandHandler, DiningCompanionQueryHandler, EnsureUserProfileCommand,
    UserDietaryContextQueryHandler, UserHealthExpectationCommandHandler,
    UserHealthExpectationQueryHandler, UserPreferencesCommandHandler, UserPreferencesQueryHandler,
    UserProfileCommandHandler, UserProfileQueryHandler,
};
use domain::UserId;
use std::sync::Arc;

use crate::config::Config as AppConfig;

const COMMON_NUTRITION_SEED: &str = include_str!("seeds/nutrition/common_foods.json");

pub struct UserState {
    pub query: UserProfileQueryHandler,
    pub command: UserProfileCommandHandler,
    pub companion_query: DiningCompanionQueryHandler,
    pub companion_command: DiningCompanionCommandHandler,
    pub dietary_context: UserDietaryContextQueryHandler,
    pub expectation_query: UserHealthExpectationQueryHandler,
    pub expectation_command: UserHealthExpectationCommandHandler,
    pub preferences_query: UserPreferencesQueryHandler,
    pub preferences_command: UserPreferencesCommandHandler,
}

pub struct MealState {
    pub command: MealCommandHandler,
    pub query: MealQueryHandler,
}

pub struct ConversationState {
    pub command: ConversationCommandHandler,
    pub query: ConversationQueryHandler,
}

pub struct AppContainer {
    pub config: AppConfig,
    pub user: UserState,
    pub meal: MealState,
    pub conversation: ConversationState,
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
        rag: RagConfig {
            lancedb_path: config.rag.lancedb_path.clone(),
            embedding_provider: embedding_route.provider,
            embedding_base_url: embedding_route.base_url,
            embedding_api_key: embedding_route.api_key,
            embedding_model: embedding_route.model,
            embedding_ndims: config.rag.embedding_ndims,
            top_k: config.rag.top_k,
        },
    };

    let user_dietary_context = UserDietaryContextQueryHandler::new(
        repos.user_repo.clone(),
        repos.user_expectation_repo.clone(),
        repos.user_preferences_repo.clone(),
        repos.dining_companion_repo.clone(),
    );

    let user = UserState {
        query: UserProfileQueryHandler::new(repos.user_repo.clone()),
        command: UserProfileCommandHandler::new(repos.user_repo.clone()),
        companion_query: DiningCompanionQueryHandler::new(repos.dining_companion_repo.clone()),
        companion_command: DiningCompanionCommandHandler::new(
            repos.dining_companion_repo.clone(),
            repos.user_repo.clone(),
        ),
        dietary_context: user_dietary_context.clone(),
        expectation_query: UserHealthExpectationQueryHandler::new(
            repos.user_expectation_repo.clone(),
        ),
        expectation_command: UserHealthExpectationCommandHandler::new(
            repos.user_expectation_repo.clone(),
            repos.user_repo.clone(),
        ),
        preferences_query: UserPreferencesQueryHandler::new(repos.user_preferences_repo.clone()),
        preferences_command: UserPreferencesCommandHandler::new(
            repos.user_preferences_repo.clone(),
            repos.user_repo.clone(),
        ),
    };

    user.command
        .ensure_profile(EnsureUserProfileCommand {
            user_id: UserId::new_unchecked("default"),
            display_name: "屋主".to_string(),
        })
        .await?;

    seed_food_nutrition_references(repos.food_nutrition_reference_repo.as_ref()).await?;

    let meal_command = Arc::new(
        MealCommandHandler::new(
            user_dietary_context,
            repos.meal_repo.clone(),
            repos.pending_meal_repo.clone(),
            repos.meal_day_finalization_repo.clone(),
            repos.meal_day_summary_repo.clone(),
        )
        .with_food_nutrition_references(repos.food_nutrition_reference_repo.clone())
        .with_event_handler(MealEventHandler::new()),
    );

    let meal = MealState {
        command: meal_command.as_ref().clone(),
        query: MealQueryHandler::new(repos.meal_repo.clone()),
    };

    let conversation = ConversationState {
        command: ConversationCommandHandler::new(Arc::new(ConversationAgentService::new(
            agent_config,
            meal_command,
            repos.chat_repo.clone(),
            user.dietary_context.clone(),
        ))),
        query: ConversationQueryHandler::new(repos.chat_repo.clone()),
    };

    Ok(AppContainer {
        config,
        user,
        meal,
        conversation,
    })
}

async fn seed_food_nutrition_references(
    repo: &dyn FoodNutritionReferenceRepositoryPort,
) -> AppResult<()> {
    let references =
        serde_json::from_str::<Vec<domain::FoodNutritionReference>>(COMMON_NUTRITION_SEED)
            .map_err(AppError::internal)?;

    for reference in references {
        repo.upsert_reference(&reference)
            .await
            .map_err(AppError::upstream)?;
    }

    Ok(())
}
