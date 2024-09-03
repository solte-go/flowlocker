use derive_more::From;
use serde::Serialize;
use crate::time;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, From)]
pub enum Error {
    RecordNotFound,
    Repository(String),
    BadQuery,

    #[from]
    SurrealDB(surrealdb::Error),

    #[from]
    Time(time::error::Error),
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
