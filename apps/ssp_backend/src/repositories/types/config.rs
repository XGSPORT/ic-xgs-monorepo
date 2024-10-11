use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};

#[derive(Debug, Default, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub struct Config {
    /// The off-chain backend principal.
    pub backend_principal: Option<Principal>,
}

impl Storable for Config {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storable_impl() {
        let config = Config {
            // a random principal
            backend_principal: Some(backend_principal()),
        };
        let serialized_config = config.to_bytes();
        let deserialized_config = Config::from_bytes(serialized_config);

        assert_eq!(config, deserialized_config);
    }

    fn backend_principal() -> Principal {
        Principal::from_text("63ubj-icu27-xedai-mj7py-uj2uw-pygtr-ckarq-owt2g-fhbcc-c4urf-tqe")
            .unwrap()
    }
}
