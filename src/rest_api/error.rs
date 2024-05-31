use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::db;
use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    BadRequest(String),
    CantParseUUID(String),
    ProcessExist(String),

    #[from]
    DB(db::error::Error),

    #[from]
    SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),

    #[from]
    SurrealDB(surrealdb::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
pub enum ClientError {
    Exist,
    NotExist,
    ServiceError,
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        match self {
            Error::ProcessExist { .. } => {
                (StatusCode::LOCKED, ClientError::Exist)
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::ServiceError,
            ),
        }
    }
}


impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(Arc::new(self));

        response
    }
}