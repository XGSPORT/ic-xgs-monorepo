use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use super::ValidateRequest;

pub const MAX_NAME_CHARS_COUNT: usize = 100;
pub const MAX_SPORT_CATEGORY_CHARS_COUNT: usize = 80;
pub const MAX_NOTES_CHARS_COUNT: usize = 500;
const KB: usize = 1024;
const MB: usize = KB * KB;
pub const MAX_FILE_BYTES_SIZE: usize = MB + (500 * KB); // 1.5 MB
pub const MAX_EXTERNAL_ID_CHARS_COUNT: usize = 100;
pub const MAX_ISSUER_FULL_NAME_CHARS_COUNT: usize = 100;
pub const MAX_ISSUER_CLUB_NAME_CHARS_COUNT: usize = 100;

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CertificateContent {
    pub name: String,
    pub issued_at: String,
    pub sport_category: String,
    pub notes: Option<String>,
    pub file_uri: Option<String>,
    pub external_id: Option<String>,
    pub issuer_full_name: Option<String>,
    pub issuer_club_name: Option<String>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub user_principal: Principal,
    pub created_at: String,
    pub content: CertificateContent,
    pub managed_user_id: Option<String>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateCertificateContentRequest {
    pub name: String,
    pub issued_at: u64,
    pub sport_category: String,
    pub notes: Option<String>,
    pub file_uri: Option<String>,
    pub external_id: Option<String>,
    pub issuer_full_name: Option<String>,
    pub issuer_club_name: Option<String>,
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct CreateCertificateRequest {
    pub user_db_id: String,
    pub content: CreateCertificateContentRequest,
    pub managed_user_db_id: Option<String>,
}

impl ValidateRequest for CreateCertificateRequest {
    fn validate(&self) -> Result<(), String> {
        let content = &self.content;

        if content.name.is_empty() {
            return Err("Title cannot be empty.".to_string());
        } else if content.name.chars().count() > MAX_NAME_CHARS_COUNT {
            return Err(format!(
                "Title cannot be longer than {} characters.",
                MAX_NAME_CHARS_COUNT
            ));
        }

        if content.sport_category.is_empty() {
            return Err("Sport category cannot be empty.".to_string());
        } else if content.sport_category.chars().count() > MAX_SPORT_CATEGORY_CHARS_COUNT {
            return Err(format!(
                "Sport category cannot be longer than {} characters.",
                MAX_SPORT_CATEGORY_CHARS_COUNT
            ));
        }

        if let Some(notes) = &content.notes {
            if notes.chars().count() > MAX_NOTES_CHARS_COUNT {
                return Err(format!(
                    "Notes cannot be longer than {} characters.",
                    MAX_NOTES_CHARS_COUNT
                ));
            }
        }

        if let Some(file_uri) = &content.file_uri {
            if file_uri.len() > MAX_FILE_BYTES_SIZE {
                return Err(format!(
                    "File bytes cannot be longer than {} bytes.",
                    MAX_FILE_BYTES_SIZE
                ));
            }
        }

        if let Some(external_id) = &content.external_id {
            if external_id.chars().count() > MAX_EXTERNAL_ID_CHARS_COUNT {
                return Err(format!(
                    "External ID cannot be longer than {} characters.",
                    MAX_EXTERNAL_ID_CHARS_COUNT
                ));
            }
        }

        if let Some(issuer_full_name) = &content.issuer_full_name {
            if issuer_full_name.chars().count() > MAX_ISSUER_FULL_NAME_CHARS_COUNT {
                return Err(format!(
                    "Issuer full name cannot be longer than {} characters.",
                    MAX_ISSUER_FULL_NAME_CHARS_COUNT
                ));
            }
        }

        if let Some(issuer_club_name) = &content.issuer_club_name {
            if issuer_club_name.chars().count() > MAX_ISSUER_CLUB_NAME_CHARS_COUNT {
                return Err(format!(
                    "Issuer club name cannot be longer than {} characters.",
                    MAX_ISSUER_CLUB_NAME_CHARS_COUNT
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct CreateCertificateResponse {
    pub id: String,
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct GetUserCertificatesRequest {
    pub user_principal: Option<Principal>,
    pub user_db_id: Option<String>,
}

impl ValidateRequest for GetUserCertificatesRequest {
    fn validate(&self) -> Result<(), String> {
        if self.user_principal.is_none() && self.user_db_id.is_none() {
            return Err("Either user_principal or user_db_id must be provided.".to_string());
        }

        if self.user_principal.is_some() && self.user_db_id.is_some() {
            return Err("Only one of user_principal or user_db_id can be provided.".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct CertificatePreviewWithId {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct GetUserCertificatesResponse {
    pub certificates: Vec<CertificatePreviewWithId>,
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct CertificateWithId {
    pub id: String,
    pub certificate_cbor_hex: String,
}

#[derive(Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct GetCertificateResponse {
    pub certificate: CertificateWithId,
    pub ic_certificate: Vec<u8>,
    pub ic_certificate_witness: Vec<u8>,
}
