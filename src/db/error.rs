pub type Result<T> = core::result::Result<T, Error>;
use derive_more::From;
use serde::Serialize;

#[derive(Debug, Serialize, From)]
pub enum Error {
    #[from]
    SurrealDB(surrealdb::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
