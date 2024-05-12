mod error;

use axum::{
    Router,
    Json,
    extract::State,
    routing::post,
};

use crate::db::Database;
use serde_json::{json, Value};

use crate::db::repository::set_new_process;
use error::{Result, Error};

pub fn routes(db: Database) -> Router {
    Router::new()
        .route("/api/start_new_lock", post(lock_new_process))
        .with_state(db)
}

async fn lock_new_process(State(db): State<Database>) -> Result<Json<Value>> {
    set_new_process(db).await?;

    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}