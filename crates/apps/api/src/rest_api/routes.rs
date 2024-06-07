use std::borrow::Cow;
use std::sync::Arc;
use axum::{extract::{State, Path, Json}, routing::{post, get}, Router};
use axum::extract::FromRequest;
use axum::response::{IntoResponse, Response};
use opentelemetry::trace::{Span, Status, Tracer};
use serde::{Deserialize, Serialize};

use crate::db::Database;
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;
use lib_core::ctx::Ctx;
use lib_core::tracing::get_global_trace;

use crate::db::repository::{set_new_process, get_process_by_id, check_running_processes};
use crate::rest_api::middleware::CtxW;
use super::error::{Result, ApiError};


// Create our own JSON extractor by wrapping `axum::Json`. This makes it easy to override the
// rejection and provide our own which formats errors to match our application.
//
// `axum::Json` responds with plain text if the input is invalid.
#[derive(FromRequest, Serialize)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
    where
        axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct GetProcess {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewProcess {
    app: String,
    process: String,
    eta: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestEndpoint {
    StartNewLock,
    GetLockedProcess,
}

pub fn routes(db: Database) -> Router {
    Router::new()
        .route("/api/start_new_lock", post(lock_new_process_handler))
        .route("/api/get_locked_process/:lock_id", post(get_locked_process))
        .with_state(db)
}

async fn lock_new_process_handler(
    State(db): State<Database>,
    ctx: CtxW,
    AppJson(payload): AppJson<NewProcess>,
    // Json(payload): Json<NewProcess>,
) -> Response {
    let ctx = ctx.0;

    let mut res = _lock_new_process(ctx, db, payload).await.into_response();
    res.extensions_mut().insert(Arc::new(RequestEndpoint::StartNewLock));

    res
}

async fn get_locked_process(
    State(db): State<Database>,
    ctx: CtxW,
    Path(lock_id): Path<Uuid>,
) -> Response {
    let ctx = ctx.0;

    let mut res = _get_locked_process(db, lock_id).await.into_response();
    res.extensions_mut().insert(Arc::new(RequestEndpoint::GetLockedProcess));

    res
}

async fn _lock_new_process(
    mut ctx: Ctx,
    db: Database,
    payload: NewProcess,
) -> Result<Json<Value>> {
    info!("Request with data {:?}", payload);

    let span_ctx = ctx.get_request_span();
    let mut span = get_global_trace("flowlocker".to_string())
        .start_with_context("lock_new_process", span_ctx);


    let running_processes = match check_running_processes(span_ctx, &db, &payload.app, &payload.process).await {
        Ok(ok) => ok,
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };

    if let Some(processes) = running_processes {
        if !processes.is_empty() {
            span.set_status(Status::Error { description: Cow::from("Process already exists".to_string()) });
            return Err(ApiError::ProcessExist("Process already exists".to_string()));
        }
    }

    let id = match set_new_process(&db, payload.app, payload.process, payload.eta).await {
        Ok(ok) => ok,
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };

    let body = Json(json!({
        "result": {
            "success": true,
            "id": id,
        }
    }));

    span.set_status(Status::Ok);

    Ok(body)
}

async fn _get_locked_process(
    db: Database,
    // Path(apps): Path<String>,
    lock_id: Uuid,
    // Path(process): Path<String>,
    // Json(payload): Json<GetProcess>,
) -> Result<Json<Value>> {
    // let request_id = match uuid::Uuid::parse_str(&lock_id) {
    //     Ok(id) => id,
    //     Err(e) => return Err(Error::CantParseUUID(e.to_string()))
    // };

    info!("Request with id {:?}", lock_id);

    // let res = get_process_by_id(db, payload.id).await;
    match get_process_by_id(&db, &lock_id.to_string()).await {
        Ok(p) => {
            let body = Json(json!({
                "result": {
                    "success": p,
                    "id": lock_id,
                }
            }));

            Ok(body)
        }
        Err(e) => {
            error!("Request completed with the error: {:?}", e);
            Err(ApiError::BadRequest(e.to_string()))
        }
    }
}