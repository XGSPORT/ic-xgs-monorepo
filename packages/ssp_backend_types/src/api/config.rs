use candid::{CandidType, Deserialize, Principal};

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub backend_principal: Option<Principal>,
}
