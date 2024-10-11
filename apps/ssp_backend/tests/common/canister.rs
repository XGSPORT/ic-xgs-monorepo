use candid::Principal;
use pocket_ic::{query_candid_as, update_candid_as, CallError, ErrorCode, UserError};
use ssp_backend_types::{
    Auth0JWKSet, Config, CreateCertificateRequest, CreateCertificateResponse,
    GetCertificateResponse, GetDelegationResponse, GetUserCertificatesRequest,
    GetUserCertificatesResponse, PrepareDelegationResponse, User,
};

use super::test_env::TestEnv;

pub fn initialize_canister(env: &TestEnv, jwks: Auth0JWKSet) {
    set_jwks(env, env.controller(), jwks).unwrap();
}

pub fn extract_trap_message(res: CallError) -> String {
    match res {
        CallError::UserError(UserError {
            code: ErrorCode::CanisterCalledTrap,
            description,
        }) => description,
        _ => panic!("expected trap"),
    }
}

pub fn prepare_delegation(
    env: &TestEnv,
    sender: Principal,
    jwt: String,
) -> Result<PrepareDelegationResponse, CallError> {
    update_candid_as(
        env.pic(),
        env.canister_id(),
        sender,
        "prepare_delegation",
        (jwt,),
    )
    .map(|(res,)| res)
}

pub fn get_delegation(
    env: &TestEnv,
    sender: Principal,
    jwt: String,
    expiration: u64,
) -> Result<GetDelegationResponse, CallError> {
    query_candid_as(
        env.pic(),
        env.canister_id(),
        sender,
        "get_delegation",
        (jwt, expiration),
    )
    .map(|(res,)| res)
}

pub fn sync_jwks(env: &TestEnv, sender: Principal) -> Result<(), CallError> {
    update_candid_as(env.pic(), env.canister_id(), sender, "sync_jwks", ()).map(|(res,)| res)
}

pub fn set_jwks(env: &TestEnv, sender: Principal, jwks: Auth0JWKSet) -> Result<(), CallError> {
    update_candid_as(env.pic(), env.canister_id(), sender, "set_jwks", (jwks,)).map(|(res,)| res)
}

pub fn get_jwks(env: &TestEnv, sender: Principal) -> Result<Option<Auth0JWKSet>, CallError> {
    query_candid_as(env.pic(), env.canister_id(), sender, "get_jwks", ()).map(|(res,)| res)
}

pub fn get_config(env: &TestEnv, sender: Principal) -> Result<Config, CallError> {
    query_candid_as(env.pic(), env.canister_id(), sender, "get_config", ()).map(|(res,)| res)
}

pub fn set_backend_principal(
    env: &TestEnv,
    sender: Principal,
    principal: Principal,
) -> Result<(), CallError> {
    update_candid_as(
        env.pic(),
        env.canister_id(),
        sender,
        "set_backend_principal",
        (principal,),
    )
    .map(|(res,)| res)
}

pub fn get_my_user(env: &TestEnv, sender: Principal) -> Result<User, CallError> {
    query_candid_as(env.pic(), env.canister_id(), sender, "get_my_user", ()).map(|(res,)| res)
}

pub fn create_certificate(
    env: &TestEnv,
    sender: Principal,
    request: CreateCertificateRequest,
) -> Result<CreateCertificateResponse, CallError> {
    update_candid_as(
        env.pic(),
        env.canister_id(),
        sender,
        "create_certificate",
        (request,),
    )
    .map(|(res,)| res)
}

pub fn get_user_certificates(
    env: &TestEnv,
    sender: Principal,
    request: GetUserCertificatesRequest,
) -> Result<GetUserCertificatesResponse, CallError> {
    query_candid_as(
        env.pic(),
        env.canister_id(),
        sender,
        "get_user_certificates",
        (request,),
    )
    .map(|(res,)| res)
}

pub fn get_certificate(
    env: &TestEnv,
    sender: Principal,
    id: String,
) -> Result<GetCertificateResponse, CallError> {
    query_candid_as(
        env.pic(),
        env.canister_id(),
        sender,
        "get_certificate",
        (id,),
    )
    .map(|(res,)| res)
}
