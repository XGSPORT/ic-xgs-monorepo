use candid::{
    types::{Type, TypeInner},
    CandidType, Deserialize,
};
use chrono::{Datelike, Timelike};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

const DATE_TIME_SIZE: u32 = 25;

impl DateTime {
    pub fn new(date_time: chrono::DateTime<chrono::Utc>) -> Result<Self, String> {
        Ok(Self(date_time.with_nanosecond(0).ok_or(&format!(
            "Failed to convert date time {:?}",
            date_time
        ))?))
    }

    pub fn from_timestamp_micros(micros: u64) -> Result<Self, String> {
        let micros = micros
            .try_into()
            .map_err(|err| format!("Failed to convert timestamp {} to micros: {}", micros, err))?;
        let dt = chrono::DateTime::from_timestamp_micros(micros).ok_or(&format!(
            "Failed to convert timestamp {} to date time",
            micros
        ))?;
        Self::new(dt)
    }

    pub fn sub(&self, duration: chrono::Duration) -> Self {
        Self(self.0 - duration)
    }

    pub fn min() -> Self {
        Self(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
    }

    pub fn max() -> Result<Self, String> {
        Ok(Self(
            chrono::DateTime::<chrono::Utc>::MAX_UTC
                .with_year(9999)
                .ok_or_else(|| "Failed to create max date time.".to_string())?,
        ))
    }

    pub fn timestamp_micros(&self) -> u64 {
        self.0.timestamp_micros().try_into().unwrap()
    }
}

impl ToString for DateTime {
    fn to_string(&self) -> String {
        self.0.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    }
}

impl CandidType for DateTime {
    fn _ty() -> Type {
        TypeInner::Text.into()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        self.to_string().idl_serialize(serializer)
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|date_time| {
                chrono::DateTime::parse_from_rfc3339(&date_time)
                    .map_err(|_| serde::de::Error::custom("Invalid date time."))
            })
            .map(|date_time| Self(date_time.into()))
    }
}

impl Storable for DateTime {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.to_string().as_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(chrono::DateTime::from_str(&String::from_utf8(bytes.into_owned()).unwrap()).unwrap())
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: DATE_TIME_SIZE,
        is_fixed_size: true,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, NaiveDate, TimeZone, Utc};
    use rstest::*;

    #[rstest]
    fn storable_impl_admin() {
        let date_time = date_time();
        let serialized_date_time = date_time.to_bytes();
        let deserialized_date_time = DateTime::from_bytes(serialized_date_time);

        assert_eq!(date_time, deserialized_date_time);
    }

    #[rstest]
    fn date_time_timestamp() {
        let (timestamp, date_string) = timestamp_micros();
        let date_time = DateTime::from_timestamp_micros(timestamp).unwrap();

        assert_eq!(date_time.to_string(), date_string);
        assert_eq!(date_time.timestamp_micros(), timestamp);
    }

    fn timestamp_micros() -> (u64, String) {
        (1706899350000000, "2024-02-02T18:42:30+00:00".to_string())
    }

    const HOUR: i32 = 3600;

    fn date_time() -> DateTime {
        let tz = FixedOffset::east_opt(5 * HOUR).unwrap();
        let date_time = NaiveDate::from_ymd_opt(2021, 12, 4)
            .unwrap()
            .and_hms_opt(10, 20, 6)
            .unwrap()
            .and_local_timezone(tz)
            .unwrap()
            .naive_local();

        DateTime::new(Utc.from_local_datetime(&date_time).unwrap()).unwrap()
    }
}
