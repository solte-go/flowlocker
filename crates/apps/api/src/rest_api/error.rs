use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use derive_more::From;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use strum_macros::Display;
use tracing::error;

use crate::{repository};
use crate::rest_api::middleware;
use crate::rest_api::routes::AppJson;

pub type Result<T> = core::result::Result<T, ApiError>;

#[derive(Debug, Display)]
pub enum ApiError {
    JsonExtractorRejection(JsonRejection),
    BadRequest(String),
    ProcessExist(String),
    CtxExt(middleware::CtxExtError),
    ReqParts(middleware::RequestInfoError),
}

#[derive(Debug)]
pub(super) enum ErrorType {
    ProcessExist
}

impl From<(ErrorType, String)> for ApiError {
    fn from(err: (ErrorType, String)) -> Self {
        Self::log_error(&err.0, &err.1);
        match err.0 {
            ErrorType::ProcessExist => {
                ApiError::ProcessExist(err.1)
            }
        }
    }
}

impl ApiError {
    fn log_error(err: &ErrorType, message: &str) {
        let span = tracing::Span::current();
        error!(parent: &span, error_type = ?err, error_message = %message);
    }
}

#[serde_as]
#[derive(Debug, Serialize, From, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    BadRequest(String),
    CantParseUUID(String),
    ProcessExist(String),

    #[from]
    Repositry(repository::error::Error),

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


impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        error!("api error: {:?}", &self);

        let (status, message) = match self {
            ApiError::JsonExtractorRejection(rejection) => {
                // This error is caused by bad user input so don't log it

                (rejection.status(), rejection.body_text())
            }
            // AppError::TimeError(err) => {
            //     // Because `TraceLayer` wraps each request in a span that contains the request
            //     // method, uri, etc we don't need to include those details here
            //     tracing::error!(%err, "error from time_library");
            //
            //     // Don't expose any details about the error to the client
            //     (
            //         StatusCode::INTERNAL_SERVER_ERROR,
            //         "Something went wrong".to_owned(),
            //     )
            // }
            ApiError::ProcessExist(e) => {
                (StatusCode::LOCKED, e.to_string())
            }
            ApiError::BadRequest(e) => {
                (StatusCode::BAD_REQUEST, e.to_string())
            }

            _ => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
        };

        (status, AppJson(ErrorResponse { message })).into_response()
    }

    // let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

    // response.extensions_mut().insert(Arc::new(self));

    // response
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonExtractorRejection(rejection)
    }
}