use std::time::UNIX_EPOCH;

use surrealdb::sql::Uuid as SUUID;

use crate::db::Database;
use crate::model::{OperationStatus, Process};
use crate::time::to_u64;

use super::error::Result;

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

    Ok(new_process_id.to_string())
}
