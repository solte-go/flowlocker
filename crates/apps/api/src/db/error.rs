pub type Result<T> = core::result::Result<T, Error>;
use crate::time;
use derive_more::From;
use serde::Serialize;

#[derive(Debug, Serialize, From)]
pub enum Error {
    RecordNotFound,
    Repository(String),
    BadQuery,

    #[from]
    Time(time::error::Error),

    #[from]
    SurrealDB(surrealdb::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
