use app::user::{UserDietaryContext, UserDietaryContextQueryHandler};
use domain::UserId;

#[derive(Clone)]
pub struct LongTermFactsMemory {
    user_contexts: UserDietaryContextQueryHandler,
}

impl LongTermFactsMemory {
    pub fn new(user_contexts: UserDietaryContextQueryHandler) -> Self {
        Self { user_contexts }
    }

    pub async fn load_profile_snapshot(&self, user_id: &UserId) -> Option<UserDietaryContext> {
        self.user_contexts.get_context(user_id).await.ok().flatten()
    }

    pub fn build_profile_context(snapshot: Option<&UserDietaryContext>) -> String {
        let Some(context) = snapshot else {
            return "当前暂无已加载的饮食偏好、忌口或健康期望，请优先根据用户本轮输入谨慎判断。"
                .to_string();
        };

        let allergies = join_or_none(&context.profile.allergies);
        let preferred_cuisines = join_or_none(
            &context
                .dietary_preferences()
                .preferred_cuisines
                .iter()
                .map(|item| item.cuisine.clone())
                .collect::<Vec<_>>(),
        );
        let avoided_cuisines = join_or_none(
            &context
                .dietary_preferences()
                .avoided_cuisines
                .iter()
                .map(|item| item.cuisine.clone())
                .collect::<Vec<_>>(),
        );
        let expectations = join_or_none(
            &context
                .active_expectations
                .iter()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>(),
        );

        format!(
            "当前权威用户画像快照：\n- 过敏原：{}\n- 饮食偏好：{}\n- 饮食忌口：{}\n- 当前健康期望：{}",
            allergies, preferred_cuisines, avoided_cuisines, expectations
        )
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "无".to_string()
    } else {
        values.join("、")
    }
}
