use std::sync::Arc;

use domain::{UserId, UserProfile};

use crate::app_error::{AppError, AppResult};
use crate::user::UserProfileRepositoryPort;

#[derive(Clone)]
pub struct UserProfileQueryHandler {
    user_profiles: Arc<dyn UserProfileRepositoryPort>,
}

impl UserProfileQueryHandler {
    pub fn new(user_profiles: Arc<dyn UserProfileRepositoryPort>) -> Self {
        Self { user_profiles }
    }

    pub async fn get_profile(&self, user_id: &UserId) -> AppResult<Option<UserProfile>> {
        self.user_profiles
            .find_profile(user_id)
            .await
            .map_err(AppError::upstream)
    }
}
