use derive_more::From;
use crate::{db, time};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    Job(String),

    #[from]
    DB(db::error::Error),

    #[from]
    Time(time::error::Error),

    #[from]
    Cron(tokio_cron_scheduler::JobSchedulerError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}