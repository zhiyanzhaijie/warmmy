pub mod context;
pub mod interaction;
pub mod meal;
pub mod memory;
pub mod model;
pub mod nutrition;
pub mod prompts;
pub mod routing;
pub mod turn;

pub use context::render_user_dietary_context;
pub use interaction::AgentInteractionRequest;
pub use meal::{
    MealIntakeCritique, MealIntakeCritiqueKind, MealIntakeFood, MealIntakeInput, MealIntakeOutput,
    MealIntakeStage,
};
pub use memory::{
    is_conflict, is_duplicate, IndexPolicy, LongTermMemoryForm, MemoryCandidate, MemoryClaim,
    MemoryExtractor, MemoryIndex, MemoryObservation, MemoryPolicy, MemoryRecord, MemoryScope,
    MemorySource, MemoryStatus, MemoryStore, NoopMemoryExtractor, PromotionDecision,
    PromotionPolicy,
};
pub use model::{AgentServiceProgress, TextModelGateway, TextPromptRequest};
pub use nutrition::{
    CuratedNutritionReference, CuratedNutritionReferenceRecord, MealNutritionEstimate,
    MealNutritionStage, NutritionCurator, NutritionKnowledgeGap, NutritionKnowledgeGapReason,
    NutritionReferenceRetriever, ResolvedMealFood,
};
pub use routing::{
    classify_by_rules, AgentRoute, AgentToolId, RouteDecision, RouteDecisionSource,
    RouteDefinition, RouteInput, RoutePlanner, RouteRegistry, RuleThenModelRoutePlanner,
};
pub use turn::{
    build_warmmy_agent_preamble, AgentTurnInput, AgentTurnLifecycle, AgentTurnPlan,
    AgentTurnPlanner, DefaultAgentTurnPlanner,
};
