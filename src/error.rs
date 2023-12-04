use std::io;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error{
    Generic(String),

    SurrealDB(surrealdb::Error)
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<surrealdb::error::Db> for Error {
    fn from(val: surrealdb::error::Db) -> Self {
        Self::Generic(val.to_string())
    }
}

impl From<surrealdb::Error> for Error {
    fn from(val: surrealdb::Error) -> Self {
        Self::Generic(val.to_string())
    }
}
impl From<crate::web::Error>  for Error {
    fn from(val: crate::web::Error) -> Self {
        Self::Generic(val.to_string())
    }
}