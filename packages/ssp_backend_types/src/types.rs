use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

pub type PublicKey = ByteBuf;
pub type SessionKey = PublicKey;
pub type UserKey = PublicKey;
pub type Timestamp = u64; // in nanos since epoch
pub type Signature = ByteBuf;

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub struct Delegation {
    pub pubkey: PublicKey,
    pub expiration: Timestamp,
    pub targets: Option<Vec<Principal>>,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub struct SignedDelegation {
    pub delegation: Delegation,
    pub signature: Signature,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub struct PrepareDelegationResponse {
    pub user_key: UserKey,
    pub expiration: Timestamp,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub enum GetDelegationResponse {
    #[serde(rename = "signed_delegation")]
    SignedDelegation(SignedDelegation),
    #[serde(rename = "no_such_delegation")]
    NoSuchDelegation,
}

#[derive(CandidType, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Auth0JWK {
    pub kty: String,
    pub r#use: String,
    pub n: String,
    pub e: String,
    pub kid: String,
    pub x5t: String,
    pub x5c: Vec<String>,
    pub alg: String,
}

#[derive(CandidType, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Auth0JWKSet {
    pub keys: Vec<Auth0JWK>,
}

impl Auth0JWKSet {
    pub fn find_key(&self, kid: &str) -> Option<&Auth0JWK> {
        self.keys.iter().find(|it| it.kid == kid)
    }
}

/// Claims set in the Auth0 action, under the `https://hasura.io/jwt/claims` key.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct HasuraJWTClaims {
    #[serde(rename = "x-hasura-default-role")]
    pub x_hasura_default_role: String,
    #[serde(rename = "x-hasura-allowed-roles")]
    pub x_hasura_allowed_roles: Vec<String>,
    #[serde(rename = "x-hasura-user-id")]
    pub x_hasura_user_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TEST_USER_ID: &str = "bbe2bdb3-c734-4bc7-af3d-068cfc231c12";

    #[test]
    fn test_hasura_jwt_claims_serialization() {
        let claims = HasuraJWTClaims {
            x_hasura_default_role: "user".to_string(),
            x_hasura_allowed_roles: vec!["user".to_string(), "admin".to_string()],
            x_hasura_user_id: TEST_USER_ID.to_string(),
        };

        let serialized = serde_json::to_value(claims).unwrap();

        assert_eq!(
            serialized,
            json!({
                "x-hasura-default-role": "user",
                "x-hasura-allowed-roles": ["user", "admin"],
                "x-hasura-user-id": TEST_USER_ID
            })
        );
    }

    #[test]
    fn test_hasura_jwt_claims_deserialization() {
        let json_str = json!({
            "x-hasura-default-role": "user",
            "x-hasura-allowed-roles": ["user", "admin"],
            "x-hasura-user-id": TEST_USER_ID
        });

        let deserialized: HasuraJWTClaims = serde_json::from_str(&json_str.to_string()).unwrap();

        assert_eq!(deserialized.x_hasura_default_role, "user");
        assert_eq!(deserialized.x_hasura_allowed_roles, vec!["user", "admin"]);
        assert_eq!(deserialized.x_hasura_user_id, TEST_USER_ID);
    }
}
