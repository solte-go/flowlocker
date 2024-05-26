use axum::{extract::{State, Json}, routing::post, Router};
use serde::{Deserialize, Serialize};

use crate::db::Database;
use serde_json::{json, Value};
use uuid;
use tracing::{error, info};

use crate::db::repository::{set_new_process,get_process_by_id};
use super::error::{Result, Error};

#[derive(Debug, Serialize, Deserialize)]
struct GetProcess {
    id: String
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
    .route("/api/get_locked_process", post(get_locked_process))
    .with_state(db)
}


async fn lock_new_process(
    State(db): State<Database>,
    Json(payload): Json<NewProcess>,
) -> Result<Json<Value>> {
    info!("Request with data {:?}", payload);

    

    let id = set_new_process(db, payload.app, payload.process, payload.eta).await?;

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
    Json(payload): Json<GetProcess>,
) -> Result<Json<Value>> {
    let request_id = match uuid::Uuid::parse_str(&payload.id){
        Ok(id) => id,
        Err(e) => return Err(Error::CantParseUUID(e.to_string()))
    };

    info!("Request with id {:?}", request_id);

    // let res = get_process_by_id(db, payload.id).await;
    match get_process_by_id(db, request_id.to_string()).await {
        Ok(p) => {
            let body = Json(json!({
                "result": {
                    "success": p,
                    "id": request_id,
                }
            }));
        
            Ok(body)
        }
        Err(e) => {
            error!("Request completed with the error: {:?}", e);
            return Err(Error::BadRequest(e.to_string()))
        }
    }
}