use candid::Principal;
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    caller, query, update,
};
use ssp_backend_types::{Auth0JWKSet, GetDelegationResponse, PrepareDelegationResponse, Timestamp};

use crate::services::{AccessControlService, DelegationService};

#[update]
async fn prepare_delegation(jwt: String) -> PrepareDelegationResponse {
    let calling_principal = caller();

    DelegationController::default()
        .prepare_delegation(calling_principal, jwt)
        .await
}

#[query]
fn get_delegation(jwt: String, expiration: Timestamp) -> GetDelegationResponse {
    let calling_principal = caller();

    DelegationController::default().get_delegation(calling_principal, jwt, expiration)
}

#[update]
async fn sync_jwks() {
    let calling_principal = caller();

    DelegationController::default()
        .sync_jwks(calling_principal)
        .await
}

#[query(hidden = true)]
fn transform_jwks_response(args: TransformArgs) -> HttpResponse {
    DelegationController::default().transform_jwks_response(args)
}

#[update]
// used in tests
fn set_jwks(jwks: Auth0JWKSet) {
    let calling_principal = caller();

    DelegationController::default().set_jwks(calling_principal, jwks);
}

#[query]
// used in tests
fn get_jwks() -> Option<Auth0JWKSet> {
    let calling_principal = caller();

    DelegationController::default().get_jwks(calling_principal)
}

#[derive(Default)]
struct DelegationController {
    access_control_service: AccessControlService,
    delegation_service: DelegationService,
}

impl DelegationController {
    async fn prepare_delegation(
        &self,
        calling_principal: Principal,
        jwt: String,
    ) -> PrepareDelegationResponse {
        self.delegation_service
            .prepare_delegation(calling_principal, jwt)
            .await
            .unwrap()
    }

    fn get_delegation(
        &self,
        calling_principal: Principal,
        jwt: String,
        expiration: Timestamp,
    ) -> GetDelegationResponse {
        self.delegation_service
            .get_delegation(calling_principal, jwt, expiration)
    }

    async fn sync_jwks(&self, calling_principal: Principal) {
        self.access_control_service
            .assert_principal_is_controller(&calling_principal)
            .unwrap();

        self.delegation_service
            .fetch_and_store_jwks()
            .await
            .unwrap()
    }

    fn set_jwks(&self, calling_principal: Principal, jwks: Auth0JWKSet) {
        self.access_control_service
            .assert_principal_is_controller(&calling_principal)
            .unwrap();

        self.delegation_service.set_jwks(jwks)
    }

    fn get_jwks(&self, calling_principal: Principal) -> Option<Auth0JWKSet> {
        self.access_control_service
            .assert_principal_is_controller(&calling_principal)
            .unwrap();

        self.delegation_service.get_jwks()
    }

    fn transform_jwks_response(&self, args: TransformArgs) -> HttpResponse {
        self.delegation_service.transform_jwks_response(args)
    }
}
