use std::fmt::Formatter;
use derive_more::From;
use crate::rest_api;
use crate::db;

pub type Result<T> = core::result::Result<T, Error>;
// pub type Error = Box<dyn std::error::Error>; // For tests and early development

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    // -- External
    #[from]
    Io(std::io::Error),

    #[from]
    API(rest_api::error::Error),

    #[from]
    DB(db::error::Error),
}


impl Error {
    pub fn custom(value: impl std::fmt::Display) -> Self {
        Self::Custom(value.to_string())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(f, "self:?")
    }
}

impl std::error::Error for Error {}