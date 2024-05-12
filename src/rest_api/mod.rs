use axum::{Json, Router};
use crate::db::Database;
use serde_json::{json, Value};

pub fn routes(mm: Database) -> Router {
    Router::new()
        .route("/api/login", post(api_login))
        .route("/api/logout", post(api_logout))
        .with_state(mm)
}

async fn lock_new_process() -> Result<Json<Value>> {
    
}