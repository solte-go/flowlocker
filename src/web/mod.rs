pub type Result<T> = core::result::Result<T, Error>;
use axum::{response::IntoResponse, http::StatusCode};
use serde::Serialize;
pub mod router;
pub mod model;

#[derive(Debug, Serialize)]
pub enum Error {
    GeneralError(String),
    Storage(surrealdb::Error)

}



impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<surrealdb::Error> for Error {
    fn from(value: surrealdb::Error) -> Self {
        Self::Storage(value)
    }
}



impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(self);

        response

    }
}