
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::get;
use chrono::{Duration,Utc};
use serde_json::{json, Value};
use crate::store::storage::Storage;
use crate::web::{Error, Result};
use super::model::Operation;




pub fn routes(mm: Storage) -> Router {
    Router::new()
        .route("/operation", get(get_operations))
        .with_state(mm)
}

async fn create_new_operation (
    State(store): State<Storage>
) -> Result<Json<Value>> {
    let _operation : Vec<Operation> = store.ds.create("operation")
    .content(Operation{
        name: "cleanup".into(),
        in_progress: false,
        timeout: Duration::seconds(5),
        started_at: Utc::now(),
        finished_at: Utc::now()+ Duration::seconds(5),
    }).await.map_err(|e|Error::GeneralError(e.to_string()))?;

    let body_response = Json(json!({
        "result": "success"
    }));

    Ok(body_response)
}

async fn get_operations(State(store): State<Storage>) -> Result<Json<Value>> {
    let _operation : Vec<Operation> = store.ds.create("operation")
        .content(Operation{
            name: "cleanup".into(),
            in_progress: false,
            timeout: Duration::seconds(5),
            started_at: Utc::now(),
            finished_at: Utc::now()+ Duration::seconds(5),
        }).await.map_err(|e|Error::GeneralError(e.to_string()))?;

    let sql = r#"
        SELECT *
        FROM type::table($table)
    "#;

    let mut groups = store.ds.query(sql)
        .bind(("table", "operation"))
        .await.map_err(|e| Error::GeneralError(e.to_string()))?;

    let op:Vec<Operation> = groups.take(0)?;

    // let data:Vec<OperationResponse> = op.into_iter()
    //     .map(|op|
    //         OperationResponse{
    //         name: op.name.to_string(),
    //         in_progress: op.in_progress,
    //         started_at: op.started_at,
    //         timeout: op.timeout,
    //     })
    //     .collect();

    let body_response = Json(json!({
        "result": op
    }));

    Ok(body_response)
}