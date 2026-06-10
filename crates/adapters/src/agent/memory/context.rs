use app::agents::render_user_dietary_context;
use app::user::UserDietaryContextQueryHandler;
use domain::UserId;

#[derive(Clone)]
pub struct MemoryContextProvider {
    contexts: UserDietaryContextQueryHandler,
}

impl MemoryContextProvider {
    pub fn new(contexts: UserDietaryContextQueryHandler) -> Self {
        Self { contexts }
    }

    pub async fn load(&self, user_id: &UserId) -> String {
        let context = self.contexts.get_context(user_id).await.ok().flatten();
        render_user_dietary_context(context.as_ref())
    }
}
