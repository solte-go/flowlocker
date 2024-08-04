use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tracing::{error, info, instrument};
use crate::db::Database;
use crate::scheduler::cleaner::Cleaner;

pub mod error;
mod cleaner;

#[derive(Debug)]
pub struct Scheduler {
    cleaner: Arc<Cleaner>,
    interval: Duration,
}

impl Scheduler {
    pub fn new(mm: Database, interval: Duration) -> Self {
        Scheduler {
            cleaner: Arc::from(Cleaner::new(mm)),
            interval,
        }
    }

    #[instrument(skip(self))]
    pub async fn start(&self) -> error::Result<()> {
        tracing::info_span!("scheduler_events");
        let rt = Arc::new(Mutex::new(Runtime::new().unwrap()));

        let rt_clone = Arc::clone(&rt);
        let cmd = Arc::clone(&self.cleaner);
        thread::spawn(move || {
            let rt = rt_clone.lock().unwrap();

            // Run the tokio interval inside the runtime
            rt.block_on(async {
                let mut ticker = tokio::time::interval(Duration::from_secs(20));

                loop {
                    ticker.tick().await;
                    let err = cmd.run().await;
                    match err {
                        Err(e) => {
                            error!("{:?}", e)
                        }
                        _ => {
                            info!(caller = "scheduler", event = "cleanup cycle completed successfully")
                        }
                    }
                }
            });
        });
        Ok(())
    }
}
