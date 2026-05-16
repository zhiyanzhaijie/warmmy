use async_trait::async_trait;
use application::common::agent::PlanningPort;

#[derive(Debug, Clone)]
pub struct TemplatePlanner {
    max_steps: usize,
}

impl Default for TemplatePlanner {
    fn default() -> Self {
        Self { max_steps: 8 }
    }
}

#[async_trait]
impl PlanningPort for TemplatePlanner {
    async fn plan(&self, goal: &str, context: &[String]) -> Result<Vec<String>, String> {
        if goal.trim().is_empty() {
            return Ok(vec!["澄清目标与约束".to_string()]);
        }

        let mut steps = vec![
            "识别任务目标与成功标准".to_string(),
            "整理现有上下文并提取关键信号".to_string(),
            "生成候选执行路径并选择最小可行路径".to_string(),
            "按顺序执行步骤并记录中间结果".to_string(),
            "校验结果质量与约束一致性".to_string(),
            "输出结论与下一步建议".to_string(),
        ];

        if context.iter().any(|item| !item.trim().is_empty()) {
            steps.insert(2, "补齐缺失信息并标记不确定性".to_string());
        }

        steps.truncate(self.max_steps);
        Ok(steps)
    }
}
