use canister_sig_util::{delegation_signature_msg, hash_bytes, CanisterSigPublicKey};
use ic_cdk::{api::management_canister::main::raw_rand, id, trap};
use ic_certification::Hash;
use ssp_backend_types::{PublicKey, Timestamp};

use crate::repositories::Salt;

pub(super) fn delegation_signature_msg_hash(pubkey: &PublicKey, expiration: Timestamp) -> Hash {
    let msg = delegation_signature_msg(pubkey, expiration, None);
    hash_bytes(msg)
}

pub(super) fn der_encode_canister_sig_key(seed: Vec<u8>) -> Vec<u8> {
    let my_canister_id = id();
    CanisterSigPublicKey::new(my_canister_id, seed).to_der()
}

/// Calls raw rand to retrieve a random salt (32 bytes).
pub(super) async fn random_salt() -> Salt {
    let res: Vec<u8> = match raw_rand().await {
        Ok((res,)) => res,
        Err((_, err)) => trap(&format!("failed to get salt: {err}")),
    };
    let salt: Salt = res[..].try_into().unwrap_or_else(|_| {
        trap(&format!(
            "expected raw randomness to be of length 32, got {}",
            res.len()
        ));
    });
    salt
}
