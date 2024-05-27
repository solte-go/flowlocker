use axum::{extract::{State, Path, Query, Json}, routing::{post, get}, Router};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

use crate::db::repository::{set_new_process, get_process_by_id, check_running_processes};
use super::error::{Result, Error};

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

pub fn new_routes(db: Database) -> Router {
    Router::new()
        .route("/api/start_new_lock", post(lock_new_process))
        .route("/api/get_locked_process/:lock_id", get(get_locked_process))
        .with_state(db)
}


async fn lock_new_process(
    State(db): State<Database>,
    Json(payload): Json<NewProcess>,
) -> Result<Json<Value>> {
    info!("Request with data {:?}", payload);

    let running_processes = check_running_processes(&db, &payload.app, &payload.process).await?;

    if let Some(processes) = running_processes {
        if !processes.is_empty() {
            // let client_status_error = Error.client_status_and_error());
            return Err(Error::ProcessExist("Process already exists".to_string()));
        }
    }


    let id = set_new_process(&db, payload.app, payload.process, payload.eta).await?;

    let body = Json(json!({
        "result": {
            "success": true,
            "id": id,
        }
    }));

    Ok(body)
}

async fn get_locked_process(
    State(db): State<Database>,
    // Path(app): Path<String>,
    Path(lock_id): Path<Uuid>,
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
            Err(Error::BadRequest(e.to_string()))
        }
    }
}