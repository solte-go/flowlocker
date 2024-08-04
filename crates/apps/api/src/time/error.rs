use derive_more::From;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, From)]
pub enum Error {
    TimeConversion,
}
