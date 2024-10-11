use std::cell::RefCell;

use canister_sig_util::signature_map::SignatureMap;
use ic_certification::Hash;
use ssp_backend_types::Auth0JWKSet;

use super::{init_salt, Salt, SaltMemory};

pub struct DelegationState {
    sigs: SignatureMap,
    jwks: Option<Auth0JWKSet>,
    salt: SaltMemory,
}

impl Default for DelegationState {
    fn default() -> Self {
        Self {
            sigs: SignatureMap::default(),
            jwks: None,
            salt: init_salt(),
        }
    }
}

thread_local! {
  static STATE: RefCell<DelegationState> = RefCell::new(DelegationState::default());
}

#[derive(Default)]
pub struct DelegationRepository {}

impl DelegationRepository {
    pub fn get_salt(&self) -> Salt {
        STATE.with_borrow(|s| s.salt.get().to_owned())
    }

    pub fn set_salt(&self, salt: Salt) {
        STATE.with_borrow_mut(|s| s.salt.set(salt).unwrap());
    }

    pub fn set_jwks(&self, jwks: Auth0JWKSet) {
        STATE.with_borrow_mut(|s| s.jwks = Some(jwks));
    }

    pub fn get_jwks(&self) -> Option<Auth0JWKSet> {
        STATE.with_borrow(|s| s.jwks.clone())
    }

    pub fn add_delegation_signature(&self, seed: &[u8], message_hash: Hash) {
        STATE.with_borrow_mut(|s| s.sigs.add_signature(seed, message_hash));
    }

    pub fn get_signature(&self, seed: &[u8], message_hash: Hash) -> Result<Vec<u8>, String> {
        STATE.with_borrow(|s| s.sigs.get_signature_as_cbor(seed, message_hash, None))
    }

    pub fn get_sigs_root_hash(&self) -> Hash {
        STATE.with_borrow(|s| s.sigs.root_hash())
    }
}
