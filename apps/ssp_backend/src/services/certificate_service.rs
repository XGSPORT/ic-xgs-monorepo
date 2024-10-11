use candid::Principal;
use ic_cdk::println;
use ssp_backend_types::{
    CreateCertificateRequest, CreateCertificateResponse, GetCertificateResponse,
    GetUserCertificatesRequest, GetUserCertificatesResponse, ValidateRequest,
};

use crate::{
    mappings::{
        map_certificate_preview_with_id, map_certificate_with_id, map_create_certificate_response,
    },
    repositories::{
        Certificate, CertificateId, CertificateRepository, DateTime,
        UserCertificateWithCertification, UserDbId, UserRepository,
    },
    system_api::get_date_time,
};

#[derive(Default)]
pub struct CertificateService {
    certificate_repository: CertificateRepository,
    user_repository: UserRepository,
}

impl CertificateService {
    pub fn get_certificate(
        &self,
        id: String,
        only_user_principal: Option<Principal>,
    ) -> Result<GetCertificateResponse, String> {
        let id = CertificateId::try_from(id.as_str())?;

        let UserCertificateWithCertification {
            certificate,
            ic_certificate,
            ic_certificate_witness,
        } = self.certificate_repository.get_certificate(&id);

        match certificate {
            Some(cert) => {
                if let Some(p) = only_user_principal {
                    if p != cert.user_principal {
                        return Err("User can only access their own certificates".to_string());
                    }
                }

                Ok(GetCertificateResponse {
                    certificate: map_certificate_with_id(id, cert),
                    ic_certificate,
                    ic_certificate_witness,
                })
            }
            None => Err("Certificate not found".to_string()),
        }
    }

    pub fn get_user_certificates(
        &self,
        request: GetUserCertificatesRequest,
        only_user_principal: Option<Principal>,
    ) -> Result<GetUserCertificatesResponse, String> {
        request.validate()?;

        let maybe_user_db_id = match request.user_db_id {
            Some(id) => Some(UserDbId::try_from(id.as_str())?),
            None => None,
        };
        let maybe_user_principal = request.user_principal.or_else(|| {
            // at this point, the user_db_id has a value because of the validation,
            // which requires either user_db_id or user_principal to be provided
            let user_db_id = maybe_user_db_id.unwrap();

            self.user_repository
                .get_user_by_db_id(&user_db_id)
                .map(|(p, _)| p)
        });

        let mut certificates = vec![];
        if let Some(user_principal) = maybe_user_principal {
            if let Some(p) = only_user_principal {
                if p != user_principal {
                    return Err("User can only access their own certificates".to_string());
                }
            }

            certificates = self
                .certificate_repository
                .get_certificates_by_user_principal(&user_principal)?;
        }

        // try getting the certificates by managed user id
        if certificates.is_empty() {
            if let Some(user_db_id) = maybe_user_db_id {
                certificates = self
                    .certificate_repository
                    .get_certificates_by_managed_user_id(&user_db_id)?;
                if let Some(p) = only_user_principal {
                    certificates.retain(|(_, cert)| cert.user_principal == p);
                }
            }
        }

        Ok(GetUserCertificatesResponse {
            certificates: certificates
                .into_iter()
                .map(|(id, certificate)| map_certificate_preview_with_id(id, certificate))
                .collect(),
        })
    }

    pub async fn create_certificate(
        &self,
        request: CreateCertificateRequest,
        calling_user_principal: Option<Principal>,
    ) -> Result<CreateCertificateResponse, String> {
        request.validate()?;

        let user_principal = match calling_user_principal {
            Some(principal) => principal,
            None => {
                let user_db_id = UserDbId::try_from(request.user_db_id.as_str())?;
                self.user_repository
                    .get_user_by_db_id(&user_db_id)
                    .ok_or_else(|| {
                        format!(
                            "User with database id {} does not exist",
                            user_db_id.to_string()
                        )
                    })?
                    .0
            }
        };

        let date_time = get_date_time()?;

        let certificate = Certificate {
            user_principal,
            created_at: DateTime::new(date_time)?,
            content: request.content.try_into()?,
            managed_user_id: match request.managed_user_db_id {
                Some(id) => Some(UserDbId::try_from(id.as_str())?),
                None => None,
            },
        };

        let id = self
            .certificate_repository
            .create_certificate(certificate)
            .await?;

        println!(
            "Created certificate for user {} with id: {}",
            user_principal.to_text(),
            id.to_string()
        );

        Ok(map_create_certificate_response(id))
    }

    pub fn certify_all_certificates(&self) {
        self.certificate_repository.certify_all_certificates();
    }
}
