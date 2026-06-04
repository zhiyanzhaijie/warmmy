#[derive(Debug, Clone, toasty::Model)]
pub struct MemoryRecordRow {
    #[key]
    pub id: String,
    #[index]
    pub user_id: String,
    pub tier: String,
    pub form: Option<String>,
    pub scope_json: String,
    pub source_json: String,
    pub status: String,
    pub index_policy: String,
    pub content: String,
    pub search_text: String,
    pub confidence: f32,
    pub claim_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
