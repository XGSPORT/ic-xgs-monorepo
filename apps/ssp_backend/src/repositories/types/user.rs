use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};

use crate::system_api::get_date_time;

use super::{DateTime, Uuid};

pub type UserPrincipal = Principal;
pub type UserSub = String;
pub type UserDbId = Uuid;

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub jwt_sub: UserSub,
    pub db_id: UserDbId,
    pub created_at: DateTime,
}

impl User {
    pub fn new(jwt_sub: UserSub, db_id: &str) -> Result<Self, String> {
        let datetime = get_date_time()?;

        Ok(Self {
            jwt_sub,
            db_id: UserDbId::try_from(db_id)?,
            created_at: DateTime::new(datetime)?,
        })
    }
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::*;

    #[rstest]
    fn storable_impl() {
        let user = user();
        let serialized_user = user.to_bytes();
        let deserialized_user = User::from_bytes(serialized_user);

        assert_eq!(user, deserialized_user);
    }

    fn user() -> User {
        User::new(
            "test_sub".to_string(),
            "8c8471a5-b91a-4b8b-9e24-219136ea2b76",
        )
        .unwrap()
    }
}
