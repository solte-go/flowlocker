use std::process;
use std::str::FromStr;
use std::time::UNIX_EPOCH;
use opentelemetry::KeyValue;
use opentelemetry::trace::{Span, Status, Tracer};

use surrealdb::sql::Uuid as SUUID;
use uuid::{self, Uuid};
use tracing::instrument;
use crate::app_tracing::get_global_trace;

use crate::db::Database;
use crate::model::{OperationStatus, Process};
use crate::time::to_u64;

use super::error::{Result, Error};

#[instrument]
pub async fn set_new_process(db: Database, app_name: String, process: String, eta: u64) -> Result<String> {
    let new_process_id = Uuid::now_v7().to_string();

    let _: Option<Process> = db
        .conn
        .create(("process", &new_process_id))
        .content(Process {
            app: app_name.into(),
            name: process.into(),
            status: OperationStatus::New,
            create_at: to_u64(UNIX_EPOCH.elapsed().unwrap()),
            complete_at: 0,
            sla: eta, // TODO Default SLA FROM CONFIG
        }).await?;

        // db.query("CREATE process:john SET name = 'John Doe', age = 25").await?.check()?;

    // let mut span = get_global_trace("Oranssi_Opettaja".to_string())
    //     .start_with_context("send_response_to_end_user", &cx);
    // span.add_event(
    //     "send response to end user",
    //     vec![KeyValue::new("happened", true)],
    // );
    // span.set_status(Status::Ok);


    Ok(new_process_id)
}

#[instrument(skip(db))]
pub async fn get_process_by_id(db: Database, id: String) -> Result<Process> {
    // let p: Option<Process> = db.conn.select(("process", id)).await?;

    let result: Option<Process> = db.conn.select(("process", id)).await?;

    println!("{:?}", result);
    

    // let mut result: surrealdb::Response = db.conn
    //     .query("SELECT * FROM type::table($table) WHERE p_id = $id")
    //     .bind(("table", "process"))
    //     .bind(("id", "018fa928-0025-7ec3-9538-77d9c3ccf780"))
    //     .await?;
    
    // if let Err(e) = result.take::<Option<Process>>(0) {
    //         println!("Failed to retrieve a entry: {e:#?}");
    // }

    // let process: Option<Process> = result.take(0)?;

    if result.is_none() {
        return Err(Error::RecordNotFound);
    }

    Ok(result.unwrap())
}