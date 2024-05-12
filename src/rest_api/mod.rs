pub mod error;

use axum::{extract::State, routing::post, Json, Router};

use crate::db::Database;
use serde_json::{json, Value};

use crate::db::repository::set_new_process;
use error::Result;

pub fn routes(db: Database) -> Router {
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

//async fn echo(body: Bytes) -> Result<String> {
//    if let Ok(string) = String::from_utf8(body.to_vec()) {
//        Ok(string)
//    } else {
//        Err(Error::BadRequest("400".to_string()))
//    }
//}
