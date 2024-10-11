mod utils;

use candid::Principal;
use canister_sig_util::{hash_bytes, signature_map::LABEL_SIG};
use ic_cdk::{
    api::{
        management_canister::http_request::{
            http_request, CanisterHttpRequestArgument, HttpMethod, HttpResponse, TransformArgs,
            TransformContext,
        },
        set_certified_data,
    },
    print, trap,
};
use ic_certification::labeled_hash;
use ic_certification::Hash;
use jsonwebtoken_rustcrypto::Algorithm;
use serde_bytes::ByteBuf;
use ssp_backend_types::{
    Auth0JWKSet, Delegation, GetDelegationResponse, PrepareDelegationResponse, SessionKey,
    SignedDelegation, Timestamp,
};

use crate::repositories::{
    decode_jwt, DelegationRepository, IdToken, User, UserDbId, UserRepository, UserSub,
    AUTH0_ISSUER, EMPTY_SALT,
};

use self::utils::{delegation_signature_msg_hash, der_encode_canister_sig_key, random_salt};

const SUBNET_SIZE: u128 = 13;
// the response should be around 3KB, so we set a limit of 10KB
const MAX_RESPONSE_BYTES: u128 = 10_000;
// there's no body in the request, so we can set a low value
const REQUEST_BYTES: u128 = 100;

#[derive(Default)]
pub struct DelegationService {
    delegation_repository: DelegationRepository,
    user_repository: UserRepository,
}

impl DelegationService {
    pub async fn ensure_salt_initialized(&self) {
        let salt = self.delegation_repository.get_salt();
        if salt == EMPTY_SALT {
            let salt = random_salt().await;
            self.delegation_repository.set_salt(salt);
        }
    }

    pub async fn fetch_and_store_jwks(&self) -> Result<(), String> {
        // Formula from https://internetcomputer.org/docs/current/developer-docs/gas-cost#special-features.
        // Parameters calculated with https://github.com/domwoe/HTTPS-Outcalls-Calculator.
        let cycles: u128 = (3_000_000 + (60_000 * SUBNET_SIZE)) * SUBNET_SIZE
            + ((400 * SUBNET_SIZE) * REQUEST_BYTES)
            + ((800 * SUBNET_SIZE) * MAX_RESPONSE_BYTES);

        let (res,) = http_request(
            CanisterHttpRequestArgument {
                url: format!("{AUTH0_ISSUER}.well-known/jwks.json"),
                method: HttpMethod::GET,
                headers: vec![],
                body: None,
                max_response_bytes: Some(MAX_RESPONSE_BYTES.try_into().unwrap()),
                transform: Some(TransformContext::from_name(
                    "transform_jwks_response".to_string(),
                    vec![],
                )),
            },
            cycles,
        )
        .await
        .map_err(|e| format!("Error fetching JWKS: {:?}", e))?;

        let jwks: Auth0JWKSet = serde_json::from_slice(&res.body)
            .map_err(|e| format!("Error parsing JWKS: {:?}", e))?;
        self.delegation_repository.set_jwks(jwks.clone());

        print(format!(
            "Fetched JWKS. JSON Web Keys available: {}",
            jwks.keys.len()
        ));

        Ok(())
    }

    pub fn transform_jwks_response(&self, args: TransformArgs) -> HttpResponse {
        let raw_response = args.response;
        // We just need to agree on the body content and content type.
        HttpResponse {
            status: 200u8.into(),
            body: raw_response.body,
            headers: raw_response
                .headers
                .into_iter()
                .filter(|header| header.name.to_lowercase() == "content-type")
                .collect(),
        }
    }

    pub async fn prepare_delegation(
        &self,
        session_principal: Principal,
        jwt: String,
    ) -> Result<PrepareDelegationResponse, String> {
        let (token, session_key) = match self.check_authorization(session_principal, jwt) {
            Ok(res) => res,
            Err(e) => {
                trap(&e);
            }
        };

        let sub = token.claims.clone().sub;
        let db_id = match token.claims.clone().user_id() {
            Some(id) => UserDbId::try_from(id.as_str()).unwrap(),
            None => return Err("User ID not found in hasura claims".to_string()),
        };
        let expiration = token.claims.expiration_timestamp_ns();

        self.ensure_salt_initialized().await;

        let user_key = self.create_delegation(&sub, session_key, expiration);
        let user_principal = self.principal_from_sub(&sub);

        let user = User::new(sub, db_id.to_string().as_str()).unwrap();
        if self
            .user_repository
            .get_user_by_principal(&user_principal)
            .is_none()
        {
            self.user_repository
                .create_user(user_principal, user)
                .unwrap();
        }

        Ok(PrepareDelegationResponse {
            user_key,
            expiration,
        })
    }

    pub fn get_delegation(
        &self,
        session_principal: Principal,
        jwt: String,
        expiration: Timestamp,
    ) -> GetDelegationResponse {
        let (token, session_key) = match self.check_authorization(session_principal, jwt) {
            Ok(res) => res,
            Err(e) => {
                trap(&e);
            }
        };

        let sub = &token.claims.sub;
        self.load_delegation(sub, session_key, expiration)
    }

    pub fn get_jwks(&self) -> Option<Auth0JWKSet> {
        self.delegation_repository.get_jwks()
    }

    pub fn set_jwks(&self, jwks: Auth0JWKSet) {
        // add an extra layer of security:
        // we can only set the jwks once
        if self.get_jwks().is_some() {
            trap("JWKS already set. Call sync_jwks to fetch the JWKS from the auth provider");
        }

        self.delegation_repository.set_jwks(jwks)
    }

    fn check_authorization(
        &self,
        caller: Principal,
        jwt: String,
    ) -> Result<(IdToken, SessionKey), String> {
        let jwks = self.delegation_repository.get_jwks();
        let token =
            decode_jwt(&jwt, Algorithm::RS256, jwks.as_ref()).map_err(|e| format!("{:?}", e))?;

        token.claims.validate().map_err(|e| format!("{:?}", e))?;

        let nonce = {
            let nonce = hex::decode(&token.claims.nonce).map_err(|e| format!("{:?}", e))?;
            ByteBuf::from(nonce)
        };
        let token_principal = Principal::self_authenticating(&nonce);
        if caller != token_principal {
            return Err("caller and token principal mismatch".to_string());
        }

        Ok((token, nonce))
    }

    fn create_delegation(
        &self,
        user_sub: &UserSub,
        session_key: SessionKey,
        expiration: Timestamp,
    ) -> ByteBuf {
        let seed = self.calculate_seed(user_sub);

        let msg_hash = delegation_signature_msg_hash(&session_key, expiration);
        self.delegation_repository
            .add_delegation_signature(&seed, msg_hash);

        self.update_root_hash();

        ByteBuf::from(der_encode_canister_sig_key(seed.to_vec()))
    }

    fn update_root_hash(&self) {
        let root_hash = self.delegation_repository.get_sigs_root_hash();
        let prefixed_root_hash = labeled_hash(LABEL_SIG, &root_hash);
        set_certified_data(&prefixed_root_hash[..]);
    }

    fn load_delegation(
        &self,
        user_sub: &UserSub,
        session_key: SessionKey,
        expiration: Timestamp,
    ) -> GetDelegationResponse {
        let message_hash = delegation_signature_msg_hash(&session_key, expiration);

        let seed = self.calculate_seed(user_sub);

        match self
            .delegation_repository
            .get_signature(&seed, message_hash)
        {
            Ok(signature) => GetDelegationResponse::SignedDelegation(SignedDelegation {
                delegation: Delegation {
                    pubkey: session_key,
                    expiration,
                    targets: None,
                },
                signature: ByteBuf::from(signature),
            }),
            Err(_) => GetDelegationResponse::NoSuchDelegation,
        }
    }

    fn principal_from_sub(&self, user_sub: &UserSub) -> Principal {
        let seed = self.calculate_seed(user_sub);
        let public_key = der_encode_canister_sig_key(seed.to_vec());
        Principal::self_authenticating(public_key)
    }

    fn calculate_seed(&self, user_sub: &UserSub) -> Hash {
        let salt = self.delegation_repository.get_salt();

        let mut blob: Vec<u8> = vec![];
        blob.push(salt.len() as u8);
        blob.extend_from_slice(&salt);

        let user_sub_blob = user_sub.bytes();
        blob.push(user_sub_blob.len() as u8);
        blob.extend(user_sub_blob);

        hash_bytes(blob)
    }
}
