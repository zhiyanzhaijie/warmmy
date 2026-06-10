pub(super) mod common;
mod companions;
mod diet_preference;
mod health_expected;
mod profile;
mod system_preference;

use api::user;
use dioxus::prelude::*;

use crate::hooks::use_IO;
use crate::providers::CurrentUserContext;
use profile::ProfileSummaryBlock;
use system_preference::SystemPreferenceBlock;

pub use companions::CompanionsBlock;
pub use diet_preference::DietPreferenceEditBlock;
pub use health_expected::HealthExpectationEditBlock;
pub use profile::ProfileEditBlock;

#[component]
pub fn MeBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let mut preference_count = use_signal(|| 0usize);
    let mut active_count = use_signal(|| 0usize);
    let mut companion_count = use_signal(|| 0usize);

    let update_preference_stats = move |preferences: user::UserPreferencesDTO| {
        preference_count
            .set(preferences.preferred_cuisines.len() + preferences.avoided_cuisines.len());
    };

    let current_user_id = (current_user.user_id)();
    let stats_user_id = current_user_id.clone();
    let stats = use_IO(move || {
        let request_user_id = stats_user_id.clone();
        async move {
            let health_expectations = user::list_health_expectations(request_user_id.clone()).await;
            let dining_companions = user::list_dining_companions(request_user_id).await;

            (
                health_expectations
                    .map(|items| items.iter().filter(|item| item.status == "active").count()),
                dining_companions.map(|items| items.len()),
            )
        }
    });

    use_effect(move || {
        if let Some((Ok(next_active_count), Ok(next_companion_count))) = stats.read().as_ref() {
            active_count.set(*next_active_count);
            companion_count.set(*next_companion_count);
        }
    });

    rsx! {
        div { class: "h-full min-h-0 overflow-hidden p-4 md:p-8",
            section { class: "relative mx-auto h-full min-h-0 w-full max-w-6xl overflow-hidden rounded-xl border border-border bg-background shadow-none",
                div { class: "relative h-full min-h-0",
                    div { class: "absolute right-4 top-4 z-20 md:right-6 md:top-6",
                        SystemPreferenceBlock {
                            user_id: current_user_id.clone(),
                            on_saved: update_preference_stats,
                        }
                    }
                    ProfileSummaryBlock {
                        user_id: current_user_id.clone(),
                        preference_count: preference_count(),
                        active_count: active_count(),
                        companion_count: companion_count(),
                    }
                }
            }
        }
    }
}
