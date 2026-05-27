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
        let companions = if context.companions.is_empty() {
            "无".to_string()
        } else {
            context
                .companions
                .iter()
                .map(|companion| {
                    let preferred = join_or_none(
                        &companion
                            .diet
                            .preferred_cuisines
                            .iter()
                            .map(|item| item.cuisine.clone())
                            .collect::<Vec<_>>(),
                    );
                    let avoided = join_or_none(
                        &companion
                            .diet
                            .avoided_cuisines
                            .iter()
                            .map(|item| item.cuisine.clone())
                            .collect::<Vec<_>>(),
                    );
                    let notes = join_or_none(&companion.health_notes);
                    format!(
                        "{}（{}）：偏好 {}；忌口 {}；健康备注 {}",
                        companion.display_name,
                        companion.relationship.as_deref().unwrap_or("关系人"),
                        preferred,
                        avoided,
                        notes
                    )
                })
                .collect::<Vec<_>>()
                .join("\n  - ")
        };

        format!(
            "当前权威用户画像快照：\n- 用户：{}（{}）\n- 饮食偏好：{}\n- 饮食忌口：{}\n- 当前健康期望：{}\n- 常一起用餐的人：{}",
            context.profile.display_name,
            context.profile.id,
            preferred_cuisines,
            avoided_cuisines,
            expectations,
            companions
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
