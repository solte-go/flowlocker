use axum::{extract::State, routing::post, Json, Router};

use crate::db::Database;
use serde_json::{json, Value};

use crate::db::repository::set_new_process;
use super::error::Result;

pub fn new_routes(db: Database) -> Router {
    Router::new()
        .route("/api/start_new_lock", post(lock_new_process))
        .with_state(db)
}

async fn lock_new_process(
    State(db): State<Database>,
) -> Result<Json<Value>> {
    let id = set_new_process(db).await?;

    let body = Json(json!({
        "result": {
            "success": true,
            "id": id,
        }
    }));

    Ok(body)
}