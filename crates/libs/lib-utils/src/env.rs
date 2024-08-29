use std::env;
use std::str::FromStr;
use std::time::Duration;
use crate::b64::b64u_decode;

pub fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::MissingEnv(name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let value = get_env(name)?;
    value.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

pub fn get_env_duration(name: &'static str) -> Result<Duration> {
    let value = get_env(name)?;
    if let Some(stripped) = value.strip_suffix("s") {
        if let Ok(seconds) = stripped.parse::<u64>() {
            return Ok(Duration::from_secs(seconds));
        }
    }
    if let Some(stripped) = value.strip_suffix("ms") {
        if let Ok(millis) = stripped.parse::<u64>() {
            return Ok(Duration::from_millis(millis));
        }
    }

    Err(Error::WrongFormat(name))
}

pub fn get_env_b64_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    b64u_decode(&get_env(name)?)
        .map_err(|_| Error::WrongFormat(name))
}

#[derive(Debug)]
pub enum Error {
    MissingEnv(&'static str),
    WrongFormat(&'static str),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

// region:    --- Error
pub type Result<T> = core::result::Result<T, Error>;

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

// endregion: --- Error
