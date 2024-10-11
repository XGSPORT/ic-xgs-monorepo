pub const NANOS_IN_SECONDS: u64 = 1_000_000_000;

fn unix_timestamp_ns() -> u64 {
    #[cfg(target_family = "wasm")]
    {
        ic_cdk::api::time()
    }

    #[cfg(not(target_family = "wasm"))]
    {
        // fixed point in time: 2024-01-01T00:00:00Z
        1704063600000000000
    }
}

/// Returns the current unix timestamp in seconds
pub fn unix_timestamp() -> u64 {
    unix_timestamp_ns() / NANOS_IN_SECONDS
}

pub fn get_date_time() -> Result<chrono::DateTime<chrono::Utc>, String> {
    let unix_ts = unix_timestamp();
    let timestamp_s: i64 = unix_ts.try_into().map_err(|_| {
        format!(
            "Failed to convert timestamp {} from nanoseconds to seconds",
            unix_ts
        )
    })?;

    chrono::DateTime::from_timestamp(timestamp_s, 0)
        .ok_or_else(|| format!("Failed to convert timestamp {} to DateTime", timestamp_s))
}
