use derive_more::From;
use crate::db;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    Job(String),

    #[from]
    DB(db::error::Error),
    #[from]
    Cron(tokio_cron_scheduler::JobSchedulerError),
}
