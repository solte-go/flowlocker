use std::sync::Arc;

use axum::{extract::{State, Path, Json}, routing::{post, get}, Router};
use axum::extract::FromRequest;
use axum::response::{IntoResponse, Response};

use serde::{Serialize};

use crate::db::Database;
use serde_json::{json, Value};
use tracing::{error, info, instrument};


use uuid::Uuid;

use crate::db::repository::{update_process_status, get_process_by_id, check_running_processes, create_new_process};
use crate::models::OperationStatus;
use super::params::{NewProcess, ProcessData, RequestEndpoint, UnlockProcess, UpdateProcess};
use super::error::{Result, ApiError, ErrorType};


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

pub fn routes(db: Database) -> Router {
    Router::new()
        .route("/api/lock_new_process", post(handle_create_new_lock))
        .route("/api/get_locked_process/:lock_id", get(handle_get_locked_process))
        .route("/api/update_process_status/:lock_id", post(handle_update_process_status))
        .route("/api/unlock_process/:lock_id", post(handle_complete_process))
        .with_state(db)
}

async fn handle_create_new_lock(
    State(db): State<Database>,
    AppJson(payload): AppJson<NewProcess>,
) -> Response {
    let mut res = _create_new_lock(db, payload).await.into_response();
    res.extensions_mut().insert(Arc::new(RequestEndpoint::StartNewLock));

    res
}

#[instrument]
async fn handle_get_locked_process(
    State(db): State<Database>,
    Path(lock_id): Path<Uuid>,
) -> Response {
    let mut res = _get_locked_process(db, lock_id).await.into_response();
    res.extensions_mut().insert(Arc::new(RequestEndpoint::GetLockedProcess));

    res
}

async fn handle_update_process_status(
    State(db): State<Database>,
    Path(lock_id): Path<Uuid>,
    AppJson(payload): AppJson<UpdateProcess>,
) -> Response {
    _update_process_status(db, lock_id.to_string(), payload).await.into_response()
}

async fn handle_complete_process(
    State(db): State<Database>,
    Path(lock_id): Path<Uuid>,
    AppJson(payload): AppJson<UnlockProcess>,
) -> Response {
    _update_process_status(db, lock_id.to_string(), payload).await.into_response()
}

#[instrument]
async fn _create_new_lock(
    // ctx: Ctx,
    db: Database,
    payload: NewProcess,
) -> Result<Json<Value>> {
    // info!("Request with data {:?}", payload);

    let running_processes = match check_running_processes(&db, &payload.app, &payload.process).await {
        Ok(ok) => ok,
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };

    if let Some(processes) = running_processes {
        if !processes.is_empty() {
            return Err(ApiError::from((ErrorType::ProcessExist, String::from("Process already exists"))));
        }
    }

    let etc = payload.eta_to_u64()?;

    let id = match create_new_process(&db, payload.app, payload.process, etc).await {
        Ok(ok) => ok,
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };

    let body = Json(json!({
        "result": {
            "success": true,
            "id": id,
        }
    }));

    Ok(body)
}

async fn _get_locked_process(
    db: Database,
    lock_id: Uuid,
) -> Result<Json<Value>> {
    info!("Request with id {:?}", lock_id);
    match get_process_by_id(&db, &lock_id.to_string()).await {
        Ok(p) => {
            let body = Json(json!({
                "result": {
                    "success": p.to_response(),
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

async fn _update_process_status<T: ProcessData>(
    db: Database,
    id: String,
    data: T,
) -> Result<Json<Value>> {
    if data.get_status() == OperationStatus::Completed || data.get_status() == OperationStatus::Outdated {
        println!("{}", data.get_status());
        return Err(ApiError::BadRequest("bad operational status".to_string()));
    }

    let _p = match get_process_by_id(&db, &id).await {
        Ok(p) => {
            if p.status.is_outdated() {
                return Err(ApiError::BadRequest("can't updated Outdated process".to_string()));
            } else {
                p
            }
        }
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };


    match update_process_status(&db, &id, data.get_status()).await {
        Ok(ok) => ok,
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };

    let body = Json(json!({
        "result": {
            "success": true,
        }
    }));

    Ok(body)
}
