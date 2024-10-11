use candid::Principal;
use ic_cdk::{caller, query, update};
use ssp_backend_types::{
    CreateCertificateRequest, CreateCertificateResponse, GetCertificateResponse,
    GetUserCertificatesRequest, GetUserCertificatesResponse,
};

use crate::services::{AccessControlService, CertificateService};

#[update]
async fn create_certificate(req: CreateCertificateRequest) -> CreateCertificateResponse {
    let calling_principal = caller();

    CertificateController::default()
        .create_certificate(calling_principal, req)
        .await
}

#[query]
fn get_user_certificates(req: GetUserCertificatesRequest) -> GetUserCertificatesResponse {
    let calling_principal = caller();

    CertificateController::default().get_user_certificates(calling_principal, req)
}

#[query]
fn get_certificate(id: String) -> GetCertificateResponse {
    let calling_principal = caller();

    CertificateController::default().get_certificate(calling_principal, id)
}

#[derive(Default)]
struct CertificateController {
    access_control_service: AccessControlService,
    certificate_service: CertificateService,
}

impl CertificateController {
    async fn create_certificate(
        &self,
        calling_principal: Principal,
        req: CreateCertificateRequest,
    ) -> CreateCertificateResponse {
        let calling_user_principal = self
            .access_control_service
            .assert_principal_is_user_or_backend(&calling_principal)
            .unwrap();

        self.certificate_service
            .create_certificate(req, calling_user_principal.cloned())
            .await
            .unwrap()
    }

    fn get_user_certificates(
        &self,
        calling_principal: Principal,
        req: GetUserCertificatesRequest,
    ) -> GetUserCertificatesResponse {
        // if the calling principal is the backend, all certificates are visible
        let only_user_principal = self
            .access_control_service
            .assert_principal_is_user_or_backend(&calling_principal)
            .unwrap();

        self.certificate_service
            .get_user_certificates(req, only_user_principal.cloned())
            .unwrap()
    }

    fn get_certificate(&self, calling_principal: Principal, id: String) -> GetCertificateResponse {
        let only_user_principal = self
            .access_control_service
            .assert_principal_is_user_or_backend(&calling_principal)
            .unwrap();

        self.certificate_service
            .get_certificate(id, only_user_principal.cloned())
            .unwrap()
    }
}
