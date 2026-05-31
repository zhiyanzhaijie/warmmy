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
            section { class: "relative mx-auto h-full min-h-0 w-full max-w-6xl overflow-hidden rounded-[2rem] border border-border bg-card shadow-none",
                div { class: "pointer-events-none absolute -right-14 -top-20 h-56 w-56 rounded-full bg-primary/10 blur-3xl" }
                div { class: "pointer-events-none absolute -bottom-24 left-8 h-64 w-64 rounded-full bg-secondary/40 blur-3xl" }
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
