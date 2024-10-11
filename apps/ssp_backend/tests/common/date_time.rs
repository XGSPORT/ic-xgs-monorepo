use std::time::SystemTime;

pub fn date_time_from_canister_time(canister_time: SystemTime) -> chrono::DateTime<chrono::Utc> {
    canister_time.into()
}

pub fn date_time_str_from_canister_time(canister_time: SystemTime) -> String {
    // same as on the canister
    date_time_from_canister_time(canister_time).to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}
