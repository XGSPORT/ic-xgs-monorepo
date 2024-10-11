use candid::{
    types::{Serializer, Type, TypeInner},
    CandidType, Deserialize,
};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use std::borrow::Cow;
use uuid::{Builder, Uuid as UuidImpl};

use crate::system_api::with_random_bytes;

const UUID_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, PartialEq, Eq)]
pub struct Uuid(UuidImpl);

impl AsRef<[u8]> for Uuid {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Uuid {
    pub async fn new() -> Result<Self, String> {
        with_random_bytes(|bytes: [u8; UUID_SIZE]| Self::from_random_bytes(bytes)).await
    }

    pub fn from_random_bytes(bytes: [u8; UUID_SIZE]) -> Self {
        Self(Builder::from_random_bytes(bytes).into_uuid())
    }

    pub fn max() -> Self {
        Self(UuidImpl::max())
    }

    pub fn min() -> Self {
        Self(UuidImpl::nil())
    }
}

impl TryFrom<&str> for Uuid {
    type Error = String;

    fn try_from(uuid: &str) -> Result<Uuid, Self::Error> {
        let uuid = UuidImpl::parse_str(uuid)
            .map_err(|_| format!("Failed to parse UUID from string: {}", uuid))?;

        Ok(Self(uuid))
    }
}

impl ToString for Uuid {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl CandidType for Uuid {
    fn _ty() -> Type {
        TypeInner::Text.into()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.to_string().idl_serialize(serializer)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|uuid| {
            Uuid::try_from(uuid.as_str()).map_err(|_| serde::de::Error::custom("Invalid UUID."))
        })
    }
}

impl Storable for Uuid {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(UuidImpl::from_bytes(bytes.into_owned().try_into().unwrap()))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: UUID_SIZE as u32,
        is_fixed_size: true,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn storable_impl() {
        let uuid = uuid();

        let serialized_uuid = uuid.to_bytes();
        let deserialized_uuid = Uuid::from_bytes(serialized_uuid);

        assert_eq!(deserialized_uuid, uuid);
    }

    #[rstest]
    fn try_from() {
        let uuid = "e645cfd2-b365-4bda-bb64-535ffa050328";

        let result = Uuid::try_from(uuid).unwrap();

        assert_eq!(result.to_string(), uuid.to_string());
    }

    #[rstest]
    fn try_from_invalid_uuid() {
        let uuid_string = "not a uuid";

        let result = Uuid::try_from(uuid_string).unwrap_err();

        assert_eq!(
            result,
            format!("Failed to parse UUID from string: {}", uuid_string)
        );
    }

    fn uuid() -> Uuid {
        Uuid::try_from("36a1174f-b789-46e4-a5d6-ef8d38cd52b9").unwrap()
    }
}
