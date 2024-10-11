use candid::{CandidType, Deserialize};

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct User {
    pub sub: String,
    pub db_id: String,
    pub created_at: String,
}
