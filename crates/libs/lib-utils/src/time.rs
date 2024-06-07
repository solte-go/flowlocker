use time::{Duration, OffsetDateTime};
use time::format_description::well_known::Rfc3339;
use crate::error::{Result,Error};

pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn format_time(time: OffsetDateTime) -> Result<String> {
    time.format(&Rfc3339).map_err(|_| Error::FailToConvertTime)
}

pub fn now_utc_plus_sec_str(sec: f64) -> Result<String> {
    let new_time = now_utc() + Duration::seconds_f64(sec);
    format_time(new_time).map_err(|_| Error::FailToConvertTime)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(moment, &Rfc3339)
        .map_err(|_| Error::FailToDateParse(moment.to_string()))
} 