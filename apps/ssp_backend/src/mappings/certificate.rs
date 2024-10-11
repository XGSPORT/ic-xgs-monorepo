use ssp_backend_types::CreateCertificateResponse;

use crate::repositories::{Certificate, CertificateContent, CertificateId, DateTime};

impl From<CertificateContent> for ssp_backend_types::CertificateContent {
    fn from(value: CertificateContent) -> Self {
        ssp_backend_types::CertificateContent {
            name: value.name,
            issued_at: value.issued_at.to_string(),
            sport_category: value.sport_category,
            notes: value.notes,
            file_uri: value.file_uri,
            external_id: value.external_id,
            issuer_full_name: value.issuer_full_name,
            issuer_club_name: value.issuer_club_name,
        }
    }
}

impl TryFrom<ssp_backend_types::CreateCertificateContentRequest> for CertificateContent {
    type Error = String;

    fn try_from(value: ssp_backend_types::CreateCertificateContentRequest) -> Result<Self, String> {
        let issued_at = DateTime::from_timestamp_micros(value.issued_at)?;
        Ok(CertificateContent {
            name: value.name,
            issued_at,
            sport_category: value.sport_category,
            notes: value.notes,
            file_uri: value.file_uri,
            external_id: value.external_id,
            issuer_full_name: value.issuer_full_name,
            issuer_club_name: value.issuer_club_name,
        })
    }
}

impl From<Certificate> for ssp_backend_types::Certificate {
    fn from(value: Certificate) -> Self {
        ssp_backend_types::Certificate {
            user_principal: value.user_principal,
            created_at: value.created_at.to_string(),
            content: value.content.into(),
            managed_user_id: value.managed_user_id.map(|id| id.to_string()),
        }
    }
}

pub fn map_create_certificate_response(certificate_id: CertificateId) -> CreateCertificateResponse {
    CreateCertificateResponse {
        id: certificate_id.to_string(),
    }
}

pub fn map_certificate_preview_with_id(
    id: CertificateId,
    certificate: Certificate,
) -> ssp_backend_types::CertificatePreviewWithId {
    ssp_backend_types::CertificatePreviewWithId {
        id: id.to_string(),
        name: certificate.content.name,
    }
}

pub fn map_certificate_with_id(
    id: CertificateId,
    certificate: Certificate,
) -> ssp_backend_types::CertificateWithId {
    ssp_backend_types::CertificateWithId {
        id: id.to_string(),
        certificate_cbor_hex: certificate.certificate_cbor_hex(),
    }
}
