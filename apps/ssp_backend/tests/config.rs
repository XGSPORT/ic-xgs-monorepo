pub mod common;

use candid::Principal;
use common::{
    canister::{extract_trap_message, get_config, set_backend_principal},
    identity::generate_random_identity,
    test_env,
};
use ic_agent::Identity;
use ssp_backend_types::Config;

#[test]
fn test_get_config_not_controller() {
    let env = test_env::create_test_env();

    let sender = generate_random_identity().sender().unwrap();

    let res = get_config(&env, sender).unwrap_err();

    assert!(extract_trap_message(res).contains("Caller is not a controller"));
}

#[test]
fn test_get_config() {
    let env = test_env::create_test_env();

    let sender = env.controller();
    let res = get_config(&env, sender).unwrap();

    assert_eq!(
        res,
        Config {
            backend_principal: None
        }
    );
}

#[test]
fn test_set_backend_principal_not_controller() {
    let env = test_env::create_test_env();

    let sender = generate_random_identity().sender().unwrap();
    let principal = generate_random_identity().sender().unwrap();

    let res = set_backend_principal(&env, sender, principal).unwrap_err();

    assert!(extract_trap_message(res).contains("Caller is not a controller"));
}

#[test]
fn test_set_backend_principal_anonymous() {
    let env = test_env::create_test_env();

    let sender = env.controller();
    let principal = Principal::anonymous();

    let res = set_backend_principal(&env, sender, principal).unwrap_err();

    assert!(extract_trap_message(res).contains("Backend principal cannot be anonymous"));
}

#[test]
fn test_set_backend_principal() {
    let env = test_env::create_test_env();

    let sender = env.controller();
    let original_config = get_config(&env, sender).unwrap();
    assert!(original_config.backend_principal.is_none());

    let principal = generate_random_identity().sender().unwrap();

    set_backend_principal(&env, sender, principal).unwrap();
    let updated_config = get_config(&env, sender).unwrap();

    assert_eq!(updated_config.backend_principal.unwrap(), principal);
}
