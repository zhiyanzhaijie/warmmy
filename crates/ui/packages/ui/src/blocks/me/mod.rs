pub(super) mod common;
mod companions;
mod diet_preference;
mod health_expected;
mod profile;
mod system_preference;

use api::user;
use dioxus::prelude::*;

use crate::blocks::CurrentUserContext;
use companions::CompanionsBlock;
use diet_preference::DietPreferenceBlock;
use health_expected::HealthExpectedBlock;
use profile::ProfileBlock;
use system_preference::SystemPreferenceBlock;

#[component]
pub fn MeBlock() -> Element {
    let current_user = use_context::<CurrentUserContext>();
    let mut preference_count = use_signal(|| 0usize);
    let mut active_count = use_signal(|| 0usize);
    let mut proposed_count = use_signal(|| 0usize);

    let update_preference_stats = move |preferences: user::UserPreferencesDTO| {
        preference_count
            .set(preferences.preferred_cuisines.len() + preferences.avoided_cuisines.len());
    };

    let update_expectation_stats = move |items: Vec<user::HealthExpectationDTO>| {
        active_count.set(items.iter().filter(|item| item.status == "active").count());
        proposed_count.set(
            items
                .iter()
                .filter(|item| item.status == "proposed")
                .count(),
        );
    };

    let current_user_id = (current_user.user_id)();

    rsx! {
        div { class: "h-full min-h-0 overflow-y-auto px-4 py-5 pb-28 md:px-8 md:py-8 md:pb-12",
            div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6",
                ProfileBlock {
                    user_id: current_user_id.clone(),
                    preference_count: preference_count(),
                    active_count: active_count(),
                    proposed_count: proposed_count(),
                }

                div { key: "{current_user_id}", class: "grid grid-cols-1 gap-6 xl:grid-cols-[0.9fr_1.1fr]",
                    div { class: "grid grid-cols-1 gap-6",
                        SystemPreferenceBlock {
                            user_id: current_user_id.clone(),
                            on_saved: update_preference_stats,
                        }
                        DietPreferenceBlock {
                            user_id: current_user_id.clone(),
                            on_saved: update_preference_stats,
                        }
                    }
                    HealthExpectedBlock {
                        user_id: current_user_id.clone(),
                        on_loaded: update_expectation_stats,
                    }
                }
                CompanionsBlock {
                    user_id: current_user_id.clone(),
                }
            }
        }
    }
}
