pub mod common;

use core::str;
use std::time::{SystemTime, UNIX_EPOCH};

use candid::Principal;
use common::{
    auth_provider::{create_jwt, initialize_auth_provider},
    canister::{
        create_certificate, extract_trap_message, get_certificate, get_delegation,
        get_user_certificates, initialize_canister, prepare_delegation, set_backend_principal,
    },
    identity::{delegated_identity_from_delegation, generate_random_identity, pk_to_hex},
    test_env::{self, upgrade_canister, TestEnv},
};
use ic_agent::{hash_tree::SubtreeLookupResult, identity::DelegatedIdentity, Identity};
use ic_certificate_verification::VerifyCertificate;
use ic_certification::{
    leaf_hash, Certificate as IcCertificate, HashTree, HashTreeNode, LookupResult,
};
use jwt_simple::prelude::*;
use ssp_backend_types::{
    Certificate, CertificateWithId, CreateCertificateContentRequest, CreateCertificateRequest,
    GetDelegationResponse, GetUserCertificatesRequest, PrepareDelegationResponse,
    MAX_EXTERNAL_ID_CHARS_COUNT, MAX_FILE_BYTES_SIZE, MAX_ISSUER_CLUB_NAME_CHARS_COUNT,
    MAX_ISSUER_FULL_NAME_CHARS_COUNT, MAX_NAME_CHARS_COUNT, MAX_NOTES_CHARS_COUNT,
    MAX_SPORT_CATEGORY_CHARS_COUNT,
};
use uuid::Uuid;

const JWT_VALID_FOR_HOURS: u64 = 10;

const TEST_USER_SUB: &str = "test_sub";
const TEST_USER_DB_ID: &str = "96b51c08-9846-40f2-8f37-a1e4421e2ba8";

const MAX_IC_CERT_TIME_OFFSET_NS: u128 = 300_000_000_000; // 5 min

fn certificate_content_request() -> CreateCertificateContentRequest {
    CreateCertificateContentRequest {
        name: "Test certificate".to_string(),
        issued_at: 1704063600000000, // 2024-01-01 00:00:00 in microseconds
        sport_category: "Swimming".to_string(),
        notes: Some("Test notes".to_string()),
        file_uri: Some(
            "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7"
                .to_string(),
        ),
        external_id: Some("123456".to_string()),
        issuer_full_name: Some("John Doe".to_string()),
        issuer_club_name: Some("Swimming club".to_string()),
    }
}

fn decode_certificate(certificate_cbor_hex: &str) -> Certificate {
    let certificate_bytes = hex::decode(certificate_cbor_hex).unwrap();
    serde_cbor::from_slice(&certificate_bytes).unwrap()
}

fn setup_config(env: &TestEnv) -> Principal {
    let sender = env.controller();
    let backend_principal = generate_random_identity().sender().unwrap();

    set_backend_principal(env, sender, backend_principal).unwrap();

    backend_principal
}

fn create_user(
    env: &TestEnv,
    auth_provider_key_pair: &RS256KeyPair,
    user_sub: &str,
    db_id: &str,
) -> DelegatedIdentity {
    let session_identity = generate_random_identity();
    let session_principal = session_identity.sender().unwrap();
    let (jwt, _) = create_jwt(
        auth_provider_key_pair,
        user_sub,
        &pk_to_hex(&session_identity.public_key().unwrap()),
        Some(db_id),
        Duration::from_hours(JWT_VALID_FOR_HOURS),
    );

    let PrepareDelegationResponse {
        expiration,
        user_key,
    } = prepare_delegation(env, session_principal, jwt.clone()).unwrap();
    let signed_delegation = match get_delegation(env, session_principal, jwt, expiration).unwrap() {
        GetDelegationResponse::SignedDelegation(delegation) => delegation,
        _ => panic!("expected GetDelegationResponse::SignedDelegation"),
    };

    // construct the delegated identity
    delegated_identity_from_delegation(user_key, session_identity, signed_delegation)
}

fn setup_user(env: &TestEnv, user_sub: &str, db_id: &str) -> DelegatedIdentity {
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(env, jwks);

    create_user(env, &auth_provider_key_pair, user_sub, db_id)
}

#[test]
fn test_create_certificate_anonymous() {
    let env = test_env::create_test_env();
    setup_config(&env);
    setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);

    let sender = Principal::anonymous();

    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: certificate_content_request(),
        managed_user_db_id: None,
    };

    let res = create_certificate(&env, sender, request).unwrap_err();

    assert!(extract_trap_message(res).contains("Caller is not the backend or a registered user"));
}

#[test]
fn test_create_certificate_user() {
    let env = test_env::create_test_env();
    setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);

    let user_principal = user_identity.sender().unwrap();

    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: certificate_content_request(),
        managed_user_db_id: None,
    };

    let res = create_certificate(&env, user_principal, request).unwrap();
    let res_certificate = get_certificate(&env, user_principal, res.id).unwrap();
    assert_eq!(
        decode_certificate(&res_certificate.certificate.certificate_cbor_hex).user_principal,
        user_principal
    );

    // check that the certificate owner is overwritten if another id is provided
    let request = CreateCertificateRequest {
        user_db_id: "83163621-9085-4e12-88bb-0b9aee290420".to_string(),
        content: certificate_content_request(),
        managed_user_db_id: None,
    };
    let res_another_user = create_certificate(&env, user_principal, request).unwrap();
    let res_certificate_another_user =
        get_certificate(&env, user_principal, res_another_user.id).unwrap();
    assert_eq!(
        decode_certificate(
            &res_certificate_another_user
                .certificate
                .certificate_cbor_hex
        )
        .user_principal,
        user_principal
    );
}

#[test]
fn test_create_certificate() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: certificate_content_request(),
        managed_user_db_id: None,
    };

    let res = create_certificate(&env, backend_principal, request).unwrap();
    let res_certificate = get_certificate(&env, backend_principal, res.id).unwrap();
    assert_eq!(
        decode_certificate(&res_certificate.certificate.certificate_cbor_hex).user_principal,
        user_principal
    );
}

#[test]
fn test_create_certificate_no_user() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);

    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: certificate_content_request(),
        managed_user_db_id: None,
    };

    let res = create_certificate(&env, backend_principal, request).unwrap_err();

    assert!(extract_trap_message(res).contains(&format!(
        "User with database id {TEST_USER_DB_ID} does not exist"
    )));
}

#[test]
fn test_create_certificate_invalid_request() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);

    // empty name
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            name: "".to_string(),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains("Title cannot be empty."));

    // too long name
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            name: "a".repeat(MAX_NAME_CHARS_COUNT + 1),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "Title cannot be longer than {} characters.",
        MAX_NAME_CHARS_COUNT
    )));

    // empty sport category
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            sport_category: "".to_string(),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains("Sport category cannot be empty."));

    // too long sport category
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            sport_category: "a".repeat(MAX_SPORT_CATEGORY_CHARS_COUNT + 1),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "Sport category cannot be longer than {} characters.",
        MAX_SPORT_CATEGORY_CHARS_COUNT
    )));

    // too long notes
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            notes: Some("a".repeat(MAX_NOTES_CHARS_COUNT + 1)),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "Notes cannot be longer than {} characters.",
        MAX_NOTES_CHARS_COUNT
    )));

    // too long file bytes
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            file_uri: Some(
                str::from_utf8(&vec![1; MAX_FILE_BYTES_SIZE + 1])
                    .unwrap()
                    .to_string(),
            ),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "File bytes cannot be longer than {} bytes.",
        MAX_FILE_BYTES_SIZE
    )));

    // too long external id
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            external_id: Some("a".repeat(MAX_EXTERNAL_ID_CHARS_COUNT + 1)),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "External ID cannot be longer than {} characters.",
        MAX_EXTERNAL_ID_CHARS_COUNT
    )));

    // too long issuer full name
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            issuer_full_name: Some("a".repeat(MAX_ISSUER_FULL_NAME_CHARS_COUNT + 1)),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "Issuer full name cannot be longer than {} characters.",
        MAX_ISSUER_FULL_NAME_CHARS_COUNT
    )));

    // too long issuer club name
    let request = CreateCertificateRequest {
        user_db_id: TEST_USER_DB_ID.to_string(),
        content: CreateCertificateContentRequest {
            issuer_club_name: Some("a".repeat(MAX_ISSUER_CLUB_NAME_CHARS_COUNT + 1)),
            ..certificate_content_request()
        },
        managed_user_db_id: None,
    };
    let res = create_certificate(&env, backend_principal, request).unwrap_err();
    assert!(extract_trap_message(res).contains(&format!(
        "Issuer club name cannot be longer than {} characters.",
        MAX_ISSUER_CLUB_NAME_CHARS_COUNT
    )));
}

fn create_test_certificate(
    env: &TestEnv,
    backend_principal: Principal,
    user_db_id: String,
) -> (String, String) {
    let content = certificate_content_request();
    let request = CreateCertificateRequest {
        user_db_id,
        content: content.clone(),
        managed_user_db_id: None,
    };

    let res = create_certificate(env, backend_principal, request).unwrap();
    (res.id, content.name)
}

fn create_test_certificate_for_managed_user(
    env: &TestEnv,
    backend_principal: Principal,
    user_db_id: String,
    managed_user_db_id: String,
) -> (String, String) {
    let content = certificate_content_request();
    let request = CreateCertificateRequest {
        user_db_id,
        content: content.clone(),
        managed_user_db_id: Some(managed_user_db_id),
    };

    let res = create_certificate(env, backend_principal, request).unwrap();
    (res.id, content.name)
}

#[test]
fn test_get_user_certificates_invalid_request() {
    let env = test_env::create_test_env();
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    // empty request
    let request = GetUserCertificatesRequest {
        user_principal: None,
        user_db_id: None,
    };
    let res = get_user_certificates(&env, user_principal, request).unwrap_err();
    assert!(
        extract_trap_message(res).contains("Either user_principal or user_db_id must be provided.")
    );

    // both user_principal and user_db_id are provided
    let request = GetUserCertificatesRequest {
        user_principal: Some(user_principal),
        user_db_id: Some(TEST_USER_DB_ID.to_string()),
    };
    let res = get_user_certificates(&env, user_principal, request).unwrap_err();
    assert!(extract_trap_message(res)
        .contains("Only one of user_principal or user_db_id can be provided."));
}

#[test]
fn test_get_user_certificates_not_authorized() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    const TEST_CERTIFICATES_COUNT: usize = 10;
    for _ in 0..TEST_CERTIFICATES_COUNT {
        create_test_certificate(&env, backend_principal, TEST_USER_DB_ID.to_string());
    }

    for sender in [
        generate_random_identity().sender().unwrap(),
        Principal::anonymous(),
    ] {
        let request = GetUserCertificatesRequest {
            user_principal: Some(user_principal),
            user_db_id: None,
        };
        let res = get_user_certificates(&env, sender, request).unwrap_err();
        assert!(
            extract_trap_message(res).contains("Caller is not the backend or a registered user")
        );
    }
}

#[test]
fn test_get_user_certificates() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    const TEST_CERTIFICATES_COUNT: usize = 10;
    let mut created_certificates_previews = vec![];
    for _ in 0..TEST_CERTIFICATES_COUNT {
        let res = create_test_certificate(&env, backend_principal, TEST_USER_DB_ID.to_string());
        created_certificates_previews.push(res);
    }

    let test = |calling_principal: Principal| {
        // by user principal
        let request = GetUserCertificatesRequest {
            user_principal: Some(user_principal),
            user_db_id: None,
        };
        let res_by_principal = get_user_certificates(&env, calling_principal, request).unwrap();
        assert_eq!(res_by_principal.certificates.len(), TEST_CERTIFICATES_COUNT);
        // check that all certificates are present
        for (id, name) in created_certificates_previews.iter() {
            let certificate = res_by_principal
                .certificates
                .iter()
                .find(|c| c.id == *id)
                .unwrap();
            assert_eq!(certificate.name, *name);
        }

        // by user db id
        let request = GetUserCertificatesRequest {
            user_principal: None,
            user_db_id: Some(TEST_USER_DB_ID.to_string()),
        };
        let res_by_id = get_user_certificates(&env, calling_principal, request).unwrap();
        assert_eq!(res_by_id.certificates.len(), TEST_CERTIFICATES_COUNT);
        // check that all certificates are present
        for (id, name) in created_certificates_previews.iter() {
            let certificate = res_by_id.certificates.iter().find(|c| c.id == *id).unwrap();
            assert_eq!(certificate.name, *name);
        }
    };

    test(user_principal);
    test(backend_principal);
    // check that certification is still valid after canister upgrade
    upgrade_canister(&env);
    test(user_principal);
    test(backend_principal);
}

#[test]
fn test_get_user_certificates_another_user() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks);
    let first_user_db_id = TEST_USER_DB_ID;
    let first_user_identity = create_user(
        &env,
        &auth_provider_key_pair,
        TEST_USER_SUB,
        first_user_db_id,
    );
    let first_user_principal = first_user_identity.sender().unwrap();

    let second_user_sub = "test_sub_2";
    let second_user_db_id = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    let second_user_identity = create_user(
        &env,
        &auth_provider_key_pair,
        second_user_sub,
        second_user_db_id,
    );

    const TEST_CERTIFICATES_COUNT: usize = 10;

    for _ in 0..TEST_CERTIFICATES_COUNT {
        create_test_certificate(&env, backend_principal, first_user_db_id.to_string());
    }

    let second_user_principal = second_user_identity.sender().unwrap();
    // by user principal
    let request = GetUserCertificatesRequest {
        user_principal: Some(first_user_principal),
        user_db_id: None,
    };
    let res_principal = get_user_certificates(&env, second_user_principal, request).unwrap_err();
    assert!(
        extract_trap_message(res_principal).contains("User can only access their own certificates")
    );

    // by user db id
    let request = GetUserCertificatesRequest {
        user_principal: None,
        user_db_id: Some(first_user_db_id.to_string()),
    };
    let res_db_id = get_user_certificates(&env, second_user_principal, request).unwrap_err();
    assert!(extract_trap_message(res_db_id).contains("User can only access their own certificates"));

    // managed user
    const MANAGED_USER_DB_ID: &str = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    create_test_certificate_for_managed_user(
        &env,
        backend_principal,
        first_user_db_id.to_string(),
        MANAGED_USER_DB_ID.to_string(),
    );
    let request = GetUserCertificatesRequest {
        user_principal: None,
        user_db_id: Some(MANAGED_USER_DB_ID.to_string()),
    };
    let res_managed = get_user_certificates(&env, second_user_principal, request).unwrap();
    assert_eq!(res_managed.certificates.len(), 0);
}

#[test]
fn test_get_user_certificates_no_user() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);

    const TEST_CERTIFICATES_COUNT: usize = 10;

    for _ in 0..TEST_CERTIFICATES_COUNT {
        create_test_certificate(&env, backend_principal, TEST_USER_DB_ID.to_string());
    }

    // by user principal
    let non_existing_user_principal = generate_random_identity().sender().unwrap();
    let request = GetUserCertificatesRequest {
        user_principal: Some(non_existing_user_principal),
        user_db_id: None,
    };
    let res_principal = get_user_certificates(&env, backend_principal, request).unwrap();
    assert_eq!(res_principal.certificates.len(), 0);

    // by user db id
    let non_existing_user_db_id = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    let request = GetUserCertificatesRequest {
        user_principal: None,
        user_db_id: Some(non_existing_user_db_id.to_string()),
    };
    let res_db_id = get_user_certificates(&env, backend_principal, request).unwrap();
    assert_eq!(res_db_id.certificates.len(), 0);
}

#[test]
fn test_get_user_certificates_managed_user() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    const MANAGED_USER_DB_ID: &str = "ccb31f93-1a16-4089-bc84-1822ae591da2";

    const TEST_CERTIFICATES_COUNT: usize = 10;
    let mut created_certificates_previews = vec![];
    for _ in 0..TEST_CERTIFICATES_COUNT {
        let res = create_test_certificate_for_managed_user(
            &env,
            backend_principal,
            TEST_USER_DB_ID.to_string(),
            MANAGED_USER_DB_ID.to_string(),
        );
        created_certificates_previews.push(res);
    }

    let test = |calling_principal: Principal| {
        let request = GetUserCertificatesRequest {
            user_principal: None,
            user_db_id: Some(MANAGED_USER_DB_ID.to_string()),
        };
        let res = get_user_certificates(&env, calling_principal, request).unwrap();
        assert_eq!(res.certificates.len(), TEST_CERTIFICATES_COUNT);
        // check that all certificates are present
        for (id, name) in created_certificates_previews.iter() {
            let certificate = res.certificates.iter().find(|c| c.id == *id).unwrap();
            assert_eq!(certificate.name, *name);
        }
    };

    test(user_principal);
    test(backend_principal);
}

#[test]
fn test_get_certificate() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    const TEST_CERTIFICATES_COUNT: usize = 10;
    let mut created_certificates_previews = vec![];
    for _ in 0..TEST_CERTIFICATES_COUNT {
        let res = create_test_certificate(&env, backend_principal, TEST_USER_DB_ID.to_string());
        created_certificates_previews.push(res);
    }

    let test = |calling_principal: Principal| {
        for (id, name) in created_certificates_previews.iter() {
            let res_by_id = get_certificate(&env, calling_principal, id.clone()).unwrap();
            let certificate = decode_certificate(&res_by_id.certificate.certificate_cbor_hex);
            assert_eq!(certificate.user_principal, user_principal);
            assert_eq!(certificate.content.name, *name);
            assert_ic_certification_is_valid(
                &env,
                res_by_id.ic_certificate,
                res_by_id.ic_certificate_witness.clone(),
            );
            assert_ic_certificate_tree_is_valid(
                res_by_id.ic_certificate_witness,
                &user_principal,
                vec![res_by_id.certificate],
            );
        }
    };

    test(user_principal);
    test(backend_principal);
}

#[test]
fn test_get_certificate_managed_user() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let user_identity = setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);
    let user_principal = user_identity.sender().unwrap();

    const MANAGED_USER_DB_ID: &str = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    let (certificate_id, name) = create_test_certificate_for_managed_user(
        &env,
        backend_principal,
        TEST_USER_DB_ID.to_string(),
        MANAGED_USER_DB_ID.to_string(),
    );

    let test = |calling_principal: Principal| {
        let res = get_certificate(&env, calling_principal, certificate_id.clone()).unwrap();
        let certificate = decode_certificate(&res.certificate.certificate_cbor_hex);
        assert_eq!(certificate.user_principal, user_principal);
        assert_eq!(certificate.content.name, name);
        assert_eq!(
            certificate.managed_user_id.clone().unwrap(),
            MANAGED_USER_DB_ID.to_string()
        );
        assert_ic_certification_is_valid(
            &env,
            res.ic_certificate,
            res.ic_certificate_witness.clone(),
        );
        assert_ic_certificate_tree_is_valid(
            res.ic_certificate_witness,
            &user_principal,
            vec![res.certificate],
        );
    };

    test(user_principal);
    test(backend_principal);
}

#[test]
fn test_get_certificate_not_found() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);

    const TEST_CERTIFICATES_COUNT: usize = 10;

    for _ in 0..TEST_CERTIFICATES_COUNT {
        create_test_certificate(&env, backend_principal, TEST_USER_DB_ID.to_string());
    }

    let non_existing_certificate_id = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    let res = get_certificate(
        &env,
        backend_principal,
        non_existing_certificate_id.to_string(),
    )
    .unwrap_err();
    assert!(extract_trap_message(res).contains("Certificate not found"));
}

#[test]
fn test_get_certificate_another_user() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    let (auth_provider_key_pair, jwks) = initialize_auth_provider();
    initialize_canister(&env, jwks);
    let first_user_db_id = TEST_USER_DB_ID;
    create_user(
        &env,
        &auth_provider_key_pair,
        TEST_USER_SUB,
        first_user_db_id,
    );

    let second_user_sub = "test_sub_2";
    let second_user_db_id = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    let second_user_identity = create_user(
        &env,
        &auth_provider_key_pair,
        second_user_sub,
        second_user_db_id,
    );

    let (certificate_id, _) =
        create_test_certificate(&env, backend_principal, first_user_db_id.to_string());

    let second_user_principal = second_user_identity.sender().unwrap();
    let res_principal = get_certificate(&env, second_user_principal, certificate_id).unwrap_err();
    assert!(
        extract_trap_message(res_principal).contains("User can only access their own certificates")
    );

    // managed user
    const MANAGED_USER_DB_ID: &str = "ccb31f93-1a16-4089-bc84-1822ae591da2";
    let (certificate_id, _) = create_test_certificate_for_managed_user(
        &env,
        backend_principal,
        first_user_db_id.to_string(),
        MANAGED_USER_DB_ID.to_string(),
    );
    let res_managed = get_certificate(&env, second_user_principal, certificate_id).unwrap_err();
    assert!(
        extract_trap_message(res_managed).contains("User can only access their own certificates")
    );
}

#[test]
fn test_get_certificate_not_authorized() {
    let env = test_env::create_test_env();
    let backend_principal = setup_config(&env);
    setup_user(&env, TEST_USER_SUB, TEST_USER_DB_ID);

    let (certificate_id, _) =
        create_test_certificate(&env, backend_principal, TEST_USER_DB_ID.to_string());

    for sender in [
        generate_random_identity().sender().unwrap(),
        Principal::anonymous(),
    ] {
        let res = get_certificate(&env, sender, certificate_id.clone()).unwrap_err();
        assert!(
            extract_trap_message(res).contains("Caller is not the backend or a registered user")
        );
    }
}

fn assert_ic_certification_is_valid(
    test_env: &TestEnv,
    ic_certificate: Vec<u8>,
    ic_certificate_witness: Vec<u8>,
) {
    let cert: IcCertificate = serde_cbor::from_slice(&ic_certificate).unwrap();
    let canister_id = test_env.canister_id();
    let canister_id_bytes = canister_id.as_slice();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    cert.verify(
        canister_id_bytes,
        test_env.root_ic_key(),
        &current_time,
        &MAX_IC_CERT_TIME_OFFSET_NS,
    )
    .unwrap();

    let tree: HashTree = serde_cbor::from_slice(&ic_certificate_witness).unwrap();
    match cert
        .tree
        .lookup_path(vec![b"canister", canister_id_bytes, b"certified_data"])
    {
        LookupResult::Found(witness) => assert_eq!(witness, tree.digest()),
        _ => panic!("expected LookupResult::Found"),
    }
}

fn assert_ic_certificate_tree_is_valid(
    ic_certificate_witness: Vec<u8>,
    user_principal: &Principal,
    certificates: Vec<CertificateWithId>,
) {
    // SSP certificates tree structure:
    // ssp_certificates
    // └── <user_principal>
    //     └── <certificate_id>
    //         └── certificate cbor data hash
    let tree: HashTree = serde_cbor::from_slice(&ic_certificate_witness).unwrap();
    for cert in certificates.iter() {
        let id = Uuid::parse_str(cert.id.as_str()).unwrap();
        match tree.lookup_subtree(vec![
            b"ssp_certificates",
            user_principal.as_ref(),
            id.as_bytes(),
        ]) {
            SubtreeLookupResult::Found(witness_cert) => {
                let certificate_cbor = hex::decode(cert.certificate_cbor_hex.clone()).unwrap();
                match witness_cert.as_ref() {
                    HashTreeNode::Leaf(value) => {
                        assert_eq!(*value, leaf_hash(&certificate_cbor));
                    }
                    _ => panic!("expected HashTreeNode::Leaf"),
                };
            }
            _ => panic!("expected LookupResult::Found"),
        }
    }
}
