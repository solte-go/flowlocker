pub mod error;

use std::time::{Duration, UNIX_EPOCH};
use chrono::DateTime;
use crate::time::error::{Error, Result};

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

pub fn from_epoch() -> Result<u64> {
    match UNIX_EPOCH.elapsed() {
        Ok(time) => Ok(to_u64(time)),
        Err(_) => Err(Error::TimeConversion)
    }
}