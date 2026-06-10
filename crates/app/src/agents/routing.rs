use std::sync::Arc;

use async_trait::async_trait;

use crate::app_error::AppResult;
use crate::user::ResolvedAIModelConfig;

use super::model::{TextModelGateway, TextPromptRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRoute {
    Chat,
    MealIntake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentToolId {
    ProposeMealLog,
    ConfirmMealLog,
    RejectMealLog,
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub route: AgentRoute,
    pub source: RouteDecisionSource,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDecisionSource {
    Rules,
    Model,
    Fallback,
}

#[derive(Clone)]
pub struct RouteInput {
    pub text: String,
    pub has_images: bool,
    pub model: ResolvedAIModelConfig,
}

#[derive(Clone, Copy)]
pub struct RouteDefinition {
    pub route: AgentRoute,
    pub name: &'static str,
    pub description: &'static str,
    pub tool_ids: &'static [AgentToolId],
}

#[derive(Clone)]
pub struct RouteRegistry {
    definitions: Arc<[RouteDefinition]>,
}

const NO_TOOLS: &[AgentToolId] = &[];
const MEAL_INTAKE_TOOLS: &[AgentToolId] = &[
    AgentToolId::ProposeMealLog,
    AgentToolId::ConfirmMealLog,
    AgentToolId::RejectMealLog,
];

impl Default for RouteRegistry {
    fn default() -> Self {
        Self {
            definitions: Arc::from([
                RouteDefinition {
                    route: AgentRoute::MealIntake,
                    name: "meal_intake",
                    description: "用户正在表达一次真实摄入、吃了/喝了具体食物、给出食物短句可能是在记录饮食、要求创建待确认用餐记录，确认/取消待确认用餐记录，或本轮图片很可能用于识别用餐。也包括用户在追问、纠正或投诉上一轮用餐记录流程没有生成待确认记录、没有落库、没有看到记录/卡片，要求你重新检查或继续处理刚才那顿饭；这类输入即使没有再次写出具体食物，也属于 meal_intake。",
                    tool_ids: MEAL_INTAKE_TOOLS,
                },
                RouteDefinition {
                    route: AgentRoute::Chat,
                    name: "chat",
                    description: "普通聊天、营养知识咨询、泛泛建议、没有具体摄入事实的问题，或只是讨论食物但不是记录自己这次吃喝。",
                    tool_ids: NO_TOOLS,
                },
            ]),
        }
    }
}

impl RouteRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn definitions(&self) -> &[RouteDefinition] {
        &self.definitions
    }

    pub fn route_name(&self, route: AgentRoute) -> &'static str {
        self.definitions
            .iter()
            .find(|definition| definition.route == route)
            .map(|definition| definition.name)
            .unwrap_or("unknown")
    }

    pub fn tool_ids(&self, route: AgentRoute) -> &'static [AgentToolId] {
        self.definitions
            .iter()
            .find(|definition| definition.route == route)
            .map(|definition| definition.tool_ids)
            .unwrap_or(NO_TOOLS)
    }

    pub fn preamble(&self) -> String {
        let options = self
            .definitions
            .iter()
            .map(|definition| format!("- {}: {}", definition.name, definition.description))
            .collect::<Vec<_>>()
            .join("\n");
        let names = self
            .definitions
            .iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>()
            .join(" 或 ");

        format!(
            r#"你是 warmmy 的对话路由器。

把用户本轮输入分类为且只分类为以下类别之一：
{options}

判断准则：
- 如果本轮语义依赖上一轮 meal 记录流程的状态、失败、补救、确认或取消，选择 meal_intake。
- 只有当用户完全没有在处理一条用餐记录，也没有在追问记录状态时，才选择 chat。

只输出一个小写英文标签：{names}。
不要解释，不要输出 JSON，不要输出标点。"#
        )
    }

    pub fn parse(&self, raw: &str) -> Option<AgentRoute> {
        let normalized = normalize_route_text(raw);
        self.definitions
            .iter()
            .find(|definition| {
                normalized == definition.name || normalized.contains(definition.name)
            })
            .map(|definition| definition.route)
            .or_else(|| parse_route_alias(&normalized))
    }
}

#[async_trait]
pub trait RoutePlanner: Send + Sync {
    async fn plan(&self, input: RouteInput) -> AppResult<RouteDecision>;
}

#[derive(Clone)]
pub struct RuleThenModelRoutePlanner {
    registry: RouteRegistry,
    text_model: Arc<dyn TextModelGateway>,
}

impl RuleThenModelRoutePlanner {
    pub fn new(registry: RouteRegistry, text_model: Arc<dyn TextModelGateway>) -> Self {
        Self {
            registry,
            text_model,
        }
    }
}

#[async_trait]
impl RoutePlanner for RuleThenModelRoutePlanner {
    async fn plan(&self, input: RouteInput) -> AppResult<RouteDecision> {
        let rule_route = classify_by_rules(&input.text, input.has_images);
        if rule_route != AgentRoute::Chat {
            return Ok(RouteDecision {
                route: rule_route,
                source: RouteDecisionSource::Rules,
                reason: format!("{} rule matched", self.registry.route_name(rule_route)),
            });
        }

        if input.text.trim().is_empty() && !input.has_images {
            return Ok(RouteDecision {
                route: AgentRoute::Chat,
                source: RouteDecisionSource::Rules,
                reason: "empty text without images".to_string(),
            });
        }

        let prompt = build_router_prompt(&input.text, input.has_images);
        let raw = self
            .text_model
            .prompt_text(TextPromptRequest {
                model: input.model,
                preamble: self.registry.preamble(),
                prompt,
            })
            .await?;
        let parsed_route = self.registry.parse(&raw);
        let route = parsed_route.unwrap_or_else(|| {
            if input.has_images {
                AgentRoute::MealIntake
            } else {
                AgentRoute::Chat
            }
        });
        let source = if parsed_route.is_some() {
            RouteDecisionSource::Model
        } else {
            RouteDecisionSource::Fallback
        };
        let reason = if source == RouteDecisionSource::Fallback {
            format!("unparseable router output: {}", raw.trim())
        } else {
            format!("router output: {}", raw.trim())
        };

        Ok(RouteDecision {
            route,
            source,
            reason,
        })
    }
}

pub fn classify_by_rules(input: &str, has_images: bool) -> AgentRoute {
    let input = input.trim();
    if is_meal_intake_continuation(input) {
        return AgentRoute::MealIntake;
    }
    if is_meal_day_summary_continuation(input) {
        return AgentRoute::Chat;
    }
    if has_images && (input.is_empty() || looks_like_meal_input(input)) {
        return AgentRoute::MealIntake;
    }
    if looks_like_meal_input(input) {
        return AgentRoute::MealIntake;
    }
    AgentRoute::Chat
}

fn looks_like_meal_input(input: &str) -> bool {
    let has_direct_marker = [
        "吃了",
        "喝了",
        "吃",
        "喝",
        "早餐",
        "午餐",
        "晚餐",
        "早饭",
        "午饭",
        "晚饭",
        "今早",
        "早上",
        "中午",
        "下午",
        "晚上",
        "夜宵",
        "刚刚",
        "刚才",
        "记录",
        "确认卡",
        "meal",
        "breakfast",
        "lunch",
        "dinner",
        "snack",
    ]
    .iter()
    .any(|needle| input.contains(needle));
    if has_direct_marker {
        return true;
    }

    let has_portion_marker = [
        "一碗", "一份", "一杯", "一个", "半个", "两片", "一片", "g", "克",
    ]
    .iter()
    .any(|needle| input.contains(needle));
    let has_foodish_suffix = [
        "饭", "面", "粉", "粥", "汤", "饼", "包", "肉", "鱼", "鸡", "蛋", "奶", "咖啡", "茶",
    ]
    .iter()
    .any(|needle| input.contains(needle));

    has_portion_marker && has_foodish_suffix
}

fn is_meal_intake_continuation(input: &str) -> bool {
    input.contains("[warmmy:internal-continuation]")
        && (input.contains("确认一条待确认用餐记录") || input.contains("取消一条待确认用餐记录"))
}

fn is_meal_day_summary_continuation(input: &str) -> bool {
    input.contains("[warmmy:internal-continuation]") && input.contains("主动敲定今天的餐食记录")
}

fn build_router_prompt(input: &str, has_images: bool) -> String {
    let visible_text = if input.trim().is_empty() {
        "(empty)"
    } else {
        input.trim()
    };
    format!("has_images: {has_images}\nuser_input:\n{visible_text}")
}

fn normalize_route_text(raw: &str) -> String {
    raw.trim()
        .trim_matches(|c: char| c == '"' || c == '\'' || c == '`' || c.is_ascii_punctuation())
        .to_ascii_lowercase()
}

fn parse_route_alias(normalized: &str) -> Option<AgentRoute> {
    match normalized {
        "meal_intake" | "meal-intake" | "meal intake" | "meal_propose" | "meal-propose"
        | "meal propose" => Some(AgentRoute::MealIntake),
        "chat" => Some(AgentRoute::Chat),
        _ if normalized.contains("meal_intake") || normalized.contains("meal_propose") => {
            Some(AgentRoute::MealIntake)
        }
        _ if normalized.contains("chat") => Some(AgentRoute::Chat),
        _ => None,
    }
}
