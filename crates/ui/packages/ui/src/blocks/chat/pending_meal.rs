use dioxus::prelude::*;

use crate::blocks::current_user_id;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::today_session_id;

use super::state::ACTIVE_SESSION_ID;
use super::stream::{append_agent_stream, append_bot_text, append_streaming_bot_slot};
use api::meal;

#[component]
pub(super) fn PendingMealCard(pending_meal: meal::PendingMealLogDTO) -> Element {
    let user_id = current_user_id();
    let session_id = ACTIVE_SESSION_ID().unwrap_or_else(today_session_id);
    let confirm_session_id = session_id.clone();
    let reject_session_id = session_id.clone();
    let mut saving = use_signal(|| false);
    let mut rejected = use_signal(|| pending_meal.status == "rejected");
    let mut confirmed = use_signal(|| pending_meal.status == "confirmed");
    let day_cycle = use_signal(|| pending_meal.day_cycle.clone());
    let mut foods = use_signal(|| pending_meal.foods.clone());
    let mut nutrition = use_signal(|| pending_meal.nutrition.clone());
    let mut previewing = use_signal(|| false);

    let update_preview = {
        let user_id = user_id.clone();
        let pending_id = pending_meal.id.clone();
        move || {
            let request_user_id = user_id.clone();
            let input = meal::ConfirmPendingMealInput {
                pending_id: pending_id.clone(),
                day_cycle: day_cycle(),
                foods: foods(),
            };
            spawn(async move {
                previewing.set(true);
                match meal::preview_pending_meal(request_user_id, input).await {
                    Ok(updated) => {
                        nutrition.set(updated.nutrition);
                    }
                    Err(err) => append_bot_text(format!("更新营养估算失败：{err}")),
                }
                previewing.set(false);
            });
        }
    };

    let confirm_meal = {
        let pending_id = pending_meal.id.clone();
        let user_id = user_id.clone();
        move |_| {
            let request_user_id = user_id.clone();
            let request_session_id = confirm_session_id.clone();
            let input = meal::ConfirmPendingMealInput {
                pending_id: pending_id.clone(),
                day_cycle: day_cycle(),
                foods: foods(),
            };
            spawn(async move {
                saving.set(true);
                let bot_id = append_streaming_bot_slot();
                match meal::confirm_pending_meal(request_user_id, request_session_id.clone(), input)
                    .await
                {
                    Ok(stream) => {
                        confirmed.set(true);
                        append_agent_stream(stream, bot_id, request_session_id).await;
                    }
                    Err(err) => append_bot_text(format!("确认用餐记录失败：{err}")),
                }
                saving.set(false);
            });
        }
    };

    let reject_meal = {
        let pending_id = pending_meal.id.clone();
        let user_id = user_id.clone();
        move |_| {
            let request_user_id = user_id.clone();
            let request_session_id = reject_session_id.clone();
            let request_pending_id = pending_id.clone();
            spawn(async move {
                saving.set(true);
                let bot_id = append_streaming_bot_slot();
                match meal::reject_pending_meal(
                    request_user_id,
                    request_session_id.clone(),
                    request_pending_id,
                )
                .await
                {
                    Ok(stream) => {
                        rejected.set(true);
                        append_agent_stream(stream, bot_id, request_session_id).await;
                    }
                    Err(err) => append_bot_text(format!("取消用餐记录失败：{err}")),
                }
                saving.set(false);
            });
        }
    };

    rsx! {
        div { class: "rounded-[1.5rem] border border-border bg-background p-4 text-sm shadow-none",
            div { class: "flex items-start justify-between gap-3",
                div {
                    div { class: "text-base font-semibold text-foreground", "请确认这次用餐记录" }
                    p { class: "mt-1 text-xs leading-relaxed text-muted-foreground", "agent 只创建了待确认记录，确认后才会写入 meal log。" }
                }
                span { class: "rounded-full border border-border px-3 py-1 text-xs text-muted-foreground", "{day_cycle}" }
            }
            div { class: "mt-4 space-y-2",
                for (index, food) in foods().into_iter().enumerate() {
                    div { key: "{pending_meal.id}:{index}", class: "grid grid-cols-[1fr_80px_80px] gap-2",
                        Input {
                            class: "rounded-xl border border-border bg-card px-3 py-2 text-sm",
                            value: food.name.clone(),
                            oninput: {
                                let update_preview = update_preview.clone();
                                move |e: FormEvent| {
                                    foods.with_mut(|items| {
                                        if let Some(item) = items.get_mut(index) {
                                            item.name = e.value();
                                        }
                                    });
                                    update_preview();
                                }
                            },
                        }
                        Input {
                            class: "rounded-xl border border-border bg-card px-3 py-2 text-sm",
                            value: food.quantity.to_string(),
                            oninput: {
                                let update_preview = update_preview.clone();
                                move |e: FormEvent| {
                                    foods.with_mut(|items| {
                                        if let Some(item) = items.get_mut(index) {
                                            item.quantity = e.value().parse::<f32>().unwrap_or(item.quantity);
                                        }
                                    });
                                    update_preview();
                                }
                            },
                        }
                        Input {
                            class: "rounded-xl border border-border bg-card px-3 py-2 text-sm",
                            value: food.unit.clone(),
                            oninput: {
                                let update_preview = update_preview.clone();
                                move |e: FormEvent| {
                                    foods.with_mut(|items| {
                                        if let Some(item) = items.get_mut(index) {
                                            item.unit = e.value();
                                        }
                                    });
                                    update_preview();
                                }
                            },
                        }
                    }
                }
            }
            div { class: "mt-4 rounded-xl border border-border bg-card px-3 py-2 text-xs leading-relaxed text-muted-foreground",
                if previewing() {
                    "正在更新估算..."
                } else {
                    "估算：{nutrition().calories:.0} kcal · 蛋白质 {nutrition().protein_g:.1}g · 碳水 {nutrition().carbs_g:.1}g · 脂肪 {nutrition().fat_g:.1}g"
                }
            }
            div { class: "mt-4 flex flex-wrap gap-2",
                Button {
                    class: "rounded-xl bg-foreground text-background",
                    disabled: saving() || confirmed() || rejected(),
                    onclick: confirm_meal,
                    if confirmed() { "已确认" } else { "确认记录" }
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    class: "rounded-xl border border-border",
                    disabled: saving() || confirmed() || rejected(),
                    onclick: reject_meal,
                    if rejected() { "已取消" } else { "取消" }
                }
            }
        }
    }
}
