use std::sync::Arc;
use std::time::Duration;
use axum::{extract::{State, Path, Json}, routing::{post, get}, Router};
use axum::extract::FromRequest;
use axum::response::{IntoResponse, Response};
use opentelemetry::trace::{Span, Status};
use serde::{Deserialize, Serialize};

use crate::db::Database;
use serde_json::{json, Value};
use tracing::{error, info, instrument, span};


use uuid::Uuid;


use crate::db::repository::{update_process_status, get_process_by_id, check_running_processes, create_new_process};
use crate::models::OperationStatus;

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


#[derive(Debug, Serialize, Deserialize)]
struct GetProcess {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewProcess {
    app: String,
    process: String,
    eta: String,
}

impl NewProcess {
    fn eta_to_u64(&self) -> Result<u64> {
        string_to_duration(self.eta.as_str())
    }
}

fn string_to_duration(s: &str) -> Result<u64> {
    let res = s.trim_end_matches('s').parse::<u64>().map_err(|_| ApiError::BadRequest("Invalid ETA format".to_string()))?;
    Ok(res)
}


#[derive(Debug, Serialize, Deserialize)]
struct UpdateProcess {
    process_id: Uuid,
    status: OperationStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestEndpoint {
    StartNewLock,
    GetLockedProcess,
}

pub fn routes(db: Database) -> Router {
    Router::new()
        .route("/api/start_new_lock", post(handle_create_new_lock))
        .route("/api/get_locked_process/:lock_id", get(handle_get_locked_process))
        .route("/api/update_process_status", post(handle_update_process_status))
        .with_state(db)
}

async fn handle_create_new_lock(
    State(db): State<Database>,
    // ctx: CtxW,
    AppJson(payload): AppJson<NewProcess>,
    // Json(payload): Json<NewProcess>,
) -> Response {
    // let ctx = ctx.0;

    let mut res = _create_new_lock(db, payload).await.into_response();
    res.extensions_mut().insert(Arc::new(RequestEndpoint::StartNewLock));

    res
}

#[instrument]
async fn handle_get_locked_process(
    State(db): State<Database>,
    // ctx: CtxW,
    Path(lock_id): Path<Uuid>,
) -> Response {
    // let ctx = ctx.0;

    let mut res = _get_locked_process(db, lock_id).await.into_response();
    res.extensions_mut().insert(Arc::new(RequestEndpoint::GetLockedProcess));

    res
}

async fn handle_update_process_status(
    State(db): State<Database>,
    AppJson(payload): AppJson<UpdateProcess>,
) -> Response {
    //TODO process logic. Can't change process from 
    _update_process_status(db, payload).await.into_response()
}

async fn _update_process_status(
    db: Database,
    payload: UpdateProcess,
) -> Result<Json<Value>> {
    let p = match get_process_by_id(&db, payload.process_id.to_string().as_str()).await {
        Ok(p) => {
            if p.status.is_staled() {
                return Err(ApiError::BadRequest("can't updated Staled process".to_string()));
            } else {
                p
            }
        }
        Err(e) => return Err(ApiError::BadRequest(e.to_string())),
    };


    match update_process_status(&db, payload.process_id.to_string(), payload.status).await {
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

    println!("{:?}", etc);

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