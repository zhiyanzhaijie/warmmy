use super::DayCycle;
use crate::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealRecorded {
    pub user_id: UserId,
    pub day_cycle: DayCycle,
}
