pub mod common;

use candid::Principal;
use ic_agent::Identity;
use jwt_simple::prelude::*;
use ssp_backend_types::{GetDelegationResponse, PrepareDelegationResponse, User};

use common::{
    auth_provider::{create_jwt, initialize_auth_provider},
    canister::{
        extract_trap_message, get_delegation, get_my_user, initialize_canister, prepare_delegation,
    },
    date_time::date_time_str_from_canister_time,
    identity::{delegated_identity_from_delegation, generate_random_identity, pk_to_hex},
    test_env::{create_test_env, upgrade_canister},
};

/// Same as on Auth0
const JWT_VALID_FOR_HOURS: u64 = 10;

const TEST_USER_SUB: &str = "test_sub";
const TEST_USER_DB_ID: &str = "fafc11f5-c784-4cbe-9fbf-207889afd519";

#[test]
fn test_get_my_user_no_user() {
    let env = create_test_env();
    let (_, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks);

    let identity = generate_random_identity();
    let res = get_my_user(&env, identity.sender().unwrap()).unwrap_err();

    assert!(extract_trap_message(res).contains("No user found"));
}

#[test]
fn test_get_my_user() {
    let env = create_test_env();
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks);

    let session_identity = generate_random_identity();
    let session_principal = session_identity.sender().unwrap();
    let (jwt, _) = create_jwt(
        &auth_provider_key_pair,
        TEST_USER_SUB,
        &pk_to_hex(&session_identity.public_key().unwrap()),
        Some(TEST_USER_DB_ID),
        Duration::from_hours(JWT_VALID_FOR_HOURS),
    );

    let PrepareDelegationResponse {
        expiration,
        user_key,
    } = prepare_delegation(&env, session_principal, jwt.clone()).unwrap();
    let signed_delegation = match get_delegation(&env, session_principal, jwt, expiration).unwrap()
    {
        GetDelegationResponse::SignedDelegation(delegation) => delegation,
        _ => panic!("expected GetDelegationResponse::SignedDelegation"),
    };

    // construct the delegated identity
    let user_identity =
        delegated_identity_from_delegation(user_key, session_identity, signed_delegation);
    let user_principal = user_identity.sender().unwrap();

    let res = get_my_user(&env, user_principal).unwrap();

    assert_eq!(
        res,
        User {
            sub: TEST_USER_SUB.to_string(),
            created_at: date_time_str_from_canister_time(env.get_canister_time()),
            db_id: TEST_USER_DB_ID.to_string(),
        }
    );
}

#[test]
fn test_get_my_user_wrong_identity() {
    let env = create_test_env();
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks);

    let session_identity = generate_random_identity();
    let session_principal = session_identity.sender().unwrap();
    let (jwt, _) = create_jwt(
        &auth_provider_key_pair,
        TEST_USER_SUB,
        &pk_to_hex(&session_identity.public_key().unwrap()),
        Some(TEST_USER_DB_ID),
        Duration::from_hours(JWT_VALID_FOR_HOURS),
    );

    let PrepareDelegationResponse { expiration, .. } =
        prepare_delegation(&env, session_principal, jwt.clone()).unwrap();
    get_delegation(&env, session_principal, jwt, expiration).unwrap();

    // use the session identity to call the method
    let res = get_my_user(&env, session_principal).unwrap_err();
    assert!(extract_trap_message(res).contains("No user found"));

    // use another identity to call the method
    let wrong_identity = generate_random_identity();
    let res = get_my_user(&env, wrong_identity.sender().unwrap()).unwrap_err();
    assert!(extract_trap_message(res).contains("No user found"));
}

#[test]
fn test_get_my_user_anonymous() {
    let env = create_test_env();
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks);

    let session_identity = generate_random_identity();
    let session_principal = session_identity.sender().unwrap();
    let (jwt, _) = create_jwt(
        &auth_provider_key_pair,
        TEST_USER_SUB,
        &pk_to_hex(&session_identity.public_key().unwrap()),
        Some(TEST_USER_DB_ID),
        Duration::from_hours(JWT_VALID_FOR_HOURS),
    );

    let PrepareDelegationResponse { expiration, .. } =
        prepare_delegation(&env, session_principal, jwt.clone()).unwrap();
    get_delegation(&env, session_principal, jwt, expiration).unwrap();

    let res = get_my_user(&env, Principal::anonymous()).unwrap_err();
    assert!(extract_trap_message(res).contains("No user found"));
}

#[test]
fn test_get_my_user_across_upgrades() {
    let env = create_test_env();
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks.clone());

    let session_identity = generate_random_identity();
    let session_principal = session_identity.sender().unwrap();
    let (jwt, _) = create_jwt(
        &auth_provider_key_pair,
        TEST_USER_SUB,
        &pk_to_hex(&session_identity.public_key().unwrap()),
        Some(TEST_USER_DB_ID),
        Duration::from_hours(JWT_VALID_FOR_HOURS),
    );

    let PrepareDelegationResponse {
        expiration,
        user_key,
    } = prepare_delegation(&env, session_principal, jwt.clone()).unwrap();
    let signed_delegation = match get_delegation(&env, session_principal, jwt, expiration).unwrap()
    {
        GetDelegationResponse::SignedDelegation(delegation) => delegation,
        _ => panic!("expected GetDelegationResponse::SignedDelegation"),
    };

    let user_identity =
        delegated_identity_from_delegation(user_key, session_identity, signed_delegation);
    let user_principal = user_identity.sender().unwrap();

    let res_before_upgrade = get_my_user(&env, user_principal).unwrap();

    // upgrade the canister
    upgrade_canister(&env);
    initialize_canister(&env, jwks);

    let res_after_upgrade = get_my_user(&env, user_principal).unwrap();

    assert_eq!(res_before_upgrade, res_after_upgrade);
}
