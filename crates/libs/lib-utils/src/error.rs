use serde::Serialize;
use std::io;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    Generic(String),
    //Time
    FailToConvertTime,
    FailToDateParse(String),

    //Base64
    FailToB64uDecode,
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(val: io::Error) -> Self {
        Self::Generic(val.to_string())
    }
}
