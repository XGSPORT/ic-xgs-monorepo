use std::{borrow::Cow, ops::RangeBounds};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_certification::{leaf, leaf_hash, AsHashTree, Hash, HashTree};
use ic_stable_structures::{
    storable::{Blob, Bound},
    Storable,
};
use serde::Serialize;

use crate::utils::cbor_serialize;

use super::{DateTime, Uuid};

pub type CertificateId = Uuid;

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CertificateContent {
    pub external_id: Option<String>,
    pub file_uri: Option<String>,
    pub issued_at: DateTime,
    pub issuer_club_name: Option<String>,
    pub issuer_full_name: Option<String>,
    pub name: String,
    pub notes: Option<String>,
    pub sport_category: String,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub content: CertificateContent,
    pub created_at: DateTime,
    pub managed_user_id: Option<Uuid>,
    pub user_principal: Principal,
}

impl Certificate {
    pub fn certificate_cbor(&self) -> Vec<u8> {
        cbor_serialize(&self).unwrap()
    }

    pub fn certificate_cbor_hex(&self) -> String {
        hex::encode(self.certificate_cbor())
    }
}

impl AsHashTree for Certificate {
    fn root_hash(&self) -> Hash {
        let serialized = self.certificate_cbor();
        leaf_hash(&serialized[..])
    }
    fn as_hash_tree(&self) -> HashTree {
        leaf(Cow::from(self.certificate_cbor()))
    }
}

impl Storable for Certificate {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CertificateUserPrincipalKey(Blob<{ Self::MAX_SIZE as usize }>);

impl CertificateUserPrincipalKey {
    const MAX_SIZE: u32 = <(Principal, CertificateId)>::BOUND.max_size();

    pub fn new(user_principal: Principal, certificate_id: CertificateId) -> Result<Self, String> {
        Ok(Self(
            Blob::try_from((user_principal, certificate_id).to_bytes().as_ref()).map_err(|_| {
                format!(
                    "Failed to convert user principal {:?} and certificate id {:?} to bytes.",
                    user_principal, certificate_id
                )
            })?,
        ))
    }
}

impl Storable for CertificateUserPrincipalKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(Blob::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: Self::MAX_SIZE,
        is_fixed_size: true,
    };
}

pub struct CertificateUserPrincipalRange {
    start_bound: CertificateUserPrincipalKey,
    end_bound: CertificateUserPrincipalKey,
}

impl CertificateUserPrincipalRange {
    pub fn new(user_principal: Principal) -> Result<Self, String> {
        Ok(Self {
            start_bound: CertificateUserPrincipalKey::new(user_principal, CertificateId::min())?,
            end_bound: CertificateUserPrincipalKey::new(user_principal, CertificateId::max())?,
        })
    }
}

impl RangeBounds<CertificateUserPrincipalKey> for CertificateUserPrincipalRange {
    fn start_bound(&self) -> std::ops::Bound<&CertificateUserPrincipalKey> {
        std::ops::Bound::Included(&self.start_bound)
    }

    fn end_bound(&self) -> std::ops::Bound<&CertificateUserPrincipalKey> {
        std::ops::Bound::Included(&self.end_bound)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CertificateManagedUserIdKey(Blob<{ Self::MAX_SIZE as usize }>);

impl CertificateManagedUserIdKey {
    const MAX_SIZE: u32 = <(Uuid, CertificateId)>::BOUND.max_size();

    pub fn new(managed_user_id: Uuid, certificate_id: CertificateId) -> Result<Self, String> {
        Ok(Self(
            Blob::try_from((managed_user_id, certificate_id).to_bytes().as_ref()).map_err(
                |_| {
                    format!(
                        "Failed to convert managed user id {:?} and certificate id {:?} to bytes.",
                        managed_user_id, certificate_id
                    )
                },
            )?,
        ))
    }
}

impl Storable for CertificateManagedUserIdKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(Blob::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: Self::MAX_SIZE,
        is_fixed_size: true,
    };
}

pub struct CertificateManagedUserIdRange {
    start_bound: CertificateManagedUserIdKey,
    end_bound: CertificateManagedUserIdKey,
}

impl CertificateManagedUserIdRange {
    pub fn new(managed_user_id: Uuid) -> Result<Self, String> {
        Ok(Self {
            start_bound: CertificateManagedUserIdKey::new(managed_user_id, CertificateId::min())?,
            end_bound: CertificateManagedUserIdKey::new(managed_user_id, CertificateId::max())?,
        })
    }
}

impl RangeBounds<CertificateManagedUserIdKey> for CertificateManagedUserIdRange {
    fn start_bound(&self) -> std::ops::Bound<&CertificateManagedUserIdKey> {
        std::ops::Bound::Included(&self.start_bound)
    }

    fn end_bound(&self) -> std::ops::Bound<&CertificateManagedUserIdKey> {
        std::ops::Bound::Included(&self.end_bound)
    }
}

#[cfg(test)]
mod test {
    use crate::system_api::get_date_time;

    use super::*;
    use rstest::*;

    #[rstest]
    fn storable_impl() {
        let certificate = certificate();
        let serialized_certificate = certificate.to_bytes();
        let deserialized_certificate = Certificate::from_bytes(serialized_certificate);

        assert_eq!(certificate, deserialized_certificate);
    }

    fn certificate() -> Certificate {
        let date_time = DateTime::new(get_date_time().unwrap()).unwrap();
        Certificate {
            user_principal: Principal::from_text(
                "63ubj-icu27-xedai-mj7py-uj2uw-pygtr-ckarq-owt2g-fhbcc-c4urf-tqe",
            )
            .unwrap(),
            created_at: date_time,
            content: CertificateContent {
                name: "name".to_string(),
                issued_at: date_time,
                sport_category: "sport_category".to_string(),
                notes: None,
                file_uri: Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA".to_string()),
                external_id: None,
                issuer_full_name: None,
                issuer_club_name: None,
            },
            managed_user_id: None,
        }
    }
}
