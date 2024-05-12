use std::time::UNIX_EPOCH;
use crate::db::Database;
use crate::model::{Process, OperationStatus};
use crate::time::to_u64;
use surrealdb::sql::Uuid as SUUID;
use crate::{Result,error};
pub async fn set_new_process(db: Database) -> Result<()>{
    let _: Vec<Process> = db.conn
    .create("process")
        .content(Process {
            p_id: SUUID::new_v7(),
            name: "test".into(),
            status: OperationStatus::New,
            create_at: to_u64(UNIX_EPOCH.elapsed().unwrap()),
            complete_at: 0,
            sla: 0, // TODO Default SLA FROM CONFIG
            }
        ).await?;
    
    Ok(())
}