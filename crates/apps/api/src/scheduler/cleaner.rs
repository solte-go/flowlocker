use crate::scheduler::error::{Result};
use tracing::{debug, info, instrument};
use crate::db::Database;
use crate::db::repository::{get_running_processes, update_process_status, delete_process_by_id};
use crate::models::OperationStatus;
use crate::time;


#[derive(Debug, Clone)]
pub struct Cleaner {
    db: Database,
}

const DEFAULT_DELETION_INTERVAL: u64 = 600;

// Create TASK abstraction
// Change behavior of Cleaner

impl Cleaner {
    pub fn new(db: Database) -> Self {
        Cleaner { db }
    }
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        let now_time = time::from_epoch()?;


        debug!(name = "job_events", status = "started");
        let processes = get_running_processes(&self.db).await?;

        if processes.is_some() {
            for p in processes.unwrap() {
                if p.status.is_canceled() || p.status.is_completed() {
                    if now_time > p.updated_at + DEFAULT_DELETION_INTERVAL {
                        delete_process_by_id(&self.db, &p.process_id).await?;
                        info!(name = "process deleted", process_id = %p.process_id);
                    }
                    continue;
                }

                if now_time > p.create_at + p.sla {
                    update_process_status(&self.db, &p.process_id, OperationStatus::Outdated).await?;
                    info!(name = "process status changed", status = OperationStatus::Outdated.to_string());
                }
            }
        }

        debug!(name = "job_events", status = "completed successfully");

        Ok(())
    }
}
