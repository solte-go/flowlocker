use std::time::UNIX_EPOCH;
use opentelemetry::KeyValue;
use opentelemetry::trace::{Span, Status, Tracer};

use surrealdb::sql::Uuid as SUUID;
use tracing::instrument;
use crate::app_tracing::get_global_trace;

use crate::db::Database;
use crate::model::{OperationStatus, Process};
use crate::time::to_u64;

use super::error::Result;

#[instrument]
pub async fn set_new_process(db: Database) -> Result<String> {
    let new_process_id = SUUID::new_v7();

    let _: Vec<Process> = db
        .conn
        .create("process")
        .content(Process {
            p_id: new_process_id,
            name: "test".into(),
            status: OperationStatus::New,
            create_at: to_u64(UNIX_EPOCH.elapsed().unwrap()),
            complete_at: 0,
            sla: 0, // TODO Default SLA FROM CONFIG
        })
        .await?;

    // let mut span = get_global_trace("Oranssi_Opettaja".to_string())
    //     .start_with_context("send_response_to_end_user", &cx);
    // span.add_event(
    //     "send response to end user",
    //     vec![KeyValue::new("happened", true)],
    // );
    // span.set_status(Status::Ok);


    Ok(new_process_id.to_string())
}
