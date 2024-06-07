use std::time::Duration;
use chrono::DateTime;

pub fn to_u64(d: Duration) -> u64 {
    d.as_secs()
}

// Create a new Timestamp from a u64
pub fn from_u64(seconds: u64) -> Duration {
    Duration::new(seconds, 0)
}

pub fn to_string_time(t: u64) -> String {
    let dt = DateTime::from_timestamp(t as i64, 0).unwrap();
    let format = dt.format("%d-%m-%Y %H:%M:%S").to_string();
    format
}