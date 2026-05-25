#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfileUpdated {
    pub user_id: super::UserId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserHealthExpectationProposed {
    pub user_id: super::UserId,
    pub expectation_id: super::HealthExpectationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserHealthExpectationConfirmed {
    pub user_id: super::UserId,
    pub expectation_id: super::HealthExpectationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPreferencesUpdated {
    pub user_id: super::UserId,
}
