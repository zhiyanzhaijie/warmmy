use dioxus::prelude::*;
use dioxus_icons::lucide::{Plus, Trash2};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::input::Input;
use crate::providers::current_user_id;

use super::state::ChatContext;
use super::stream::{
    active_session_id, append_agent_stream, append_bot_text, append_streaming_bot_slot,
    DEFAULT_STREAM_IDLE_TIMEOUT,
};
use api::meal;

#[component]
pub(super) fn PendingMealCard(pending_meal: meal::PendingMealLogDTO) -> Element {
    let chat_state = use_context::<ChatContext>();
    let user_id = current_user_id();
    let session_id = active_session_id(chat_state);
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
                    Err(err) => append_bot_text(
                        chat_state,
                        active_session_id(chat_state),
                        format!("更新营养估算失败：{err}"),
                    ),
                }
                previewing.set(false);
            });
        }
    };

    let add_food = {
        let update_preview = update_preview.clone();
        move |_| {
            foods.with_mut(|items| {
                items.push(meal::FoodItemDTO {
                    name: String::new(),
                    quantity: 100.0,
                    unit: "g".to_string(),
                    estimated_grams: Some(100.0),
                    amount_confidence: Some(0.4),
                });
            });
            update_preview();
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
                let bot_id = append_streaming_bot_slot(chat_state, request_session_id.clone());
                match meal::confirm_pending_meal(request_user_id, request_session_id.clone(), input)
                    .await
                {
                    Ok(stream) => {
                        confirmed.set(true);
                        append_agent_stream(
                            chat_state,
                            stream,
                            bot_id,
                            request_session_id,
                            DEFAULT_STREAM_IDLE_TIMEOUT,
                        )
                        .await;
                    }
                    Err(err) => append_bot_text(
                        chat_state,
                        request_session_id,
                        format!("确认用餐记录失败：{err}"),
                    ),
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
                let bot_id = append_streaming_bot_slot(chat_state, request_session_id.clone());
                match meal::reject_pending_meal(
                    request_user_id,
                    request_session_id.clone(),
                    request_pending_id,
                )
                .await
                {
                    Ok(stream) => {
                        rejected.set(true);
                        append_agent_stream(
                            chat_state,
                            stream,
                            bot_id,
                            request_session_id,
                            DEFAULT_STREAM_IDLE_TIMEOUT,
                        )
                        .await;
                    }
                    Err(err) => append_bot_text(
                        chat_state,
                        request_session_id,
                        format!("取消用餐记录失败：{err}"),
                    ),
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
            div { class: "mt-4 space-y-2 overflow-hidden",
                for (index, food) in foods().into_iter().enumerate() {
                    div { key: "{pending_meal.id}:{index}", class: "grid min-w-0 grid-cols-[minmax(0,1fr)_104px_34px] items-center gap-2",
                        Input {
                            class: "min-w-0 rounded-xl border border-border bg-card px-3 py-2 text-sm",
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
                        div { class: "flex min-w-0 items-center rounded-xl border border-border bg-card",
                            Input {
                                class: "min-w-0 flex-1 border-0 bg-transparent px-3 py-2 text-sm",
                                value: food_grams(&food).to_string(),
                                oninput: {
                                    let update_preview = update_preview.clone();
                                    move |e: FormEvent| {
                                        foods.with_mut(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                let grams = e.value().parse::<f32>().unwrap_or_else(|_| food_grams(item));
                                                item.quantity = grams;
                                                item.unit = "g".to_string();
                                                item.estimated_grams = Some(grams);
                                            }
                                        });
                                        update_preview();
                                    }
                                },
                            }
                            span { class: "shrink-0 pr-3 text-xs text-muted-foreground", "g" }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "rounded-xl border border-border text-muted-foreground",
                            disabled: saving() || confirmed() || rejected() || foods().len() <= 1,
                            onclick: {
                                let update_preview = update_preview.clone();
                                move |_| {
                                    foods.with_mut(|items| {
                                        if items.len() > 1 && index < items.len() {
                                            items.remove(index);
                                        }
                                    });
                                    update_preview();
                                }
                            },
                            Trash2 { size: 15 }
                        }
                    }
                }
            }
            div { class: "mt-3",
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "rounded-xl border border-border px-3",
                    disabled: saving() || confirmed() || rejected(),
                    onclick: add_food,
                    Plus { size: 15 }
                    "新增食物"
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

fn food_grams(food: &meal::FoodItemDTO) -> f32 {
    food.estimated_grams.unwrap_or(food.quantity).max(0.0)
}
