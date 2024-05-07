mod db;
mod error;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::time::{Duration, UNIX_EPOCH};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::sql::Uuid as SUUID;

pub use self::error::{Error, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use surrealdb::engine::local::Db;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

//use surrealdb::engine::local::Mem; uncomment after movibng to in memory DB

//static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

#[derive(Serialize, Deserialize, Debug)]
enum OperataionStatus {
    New,
    InProgress,
    Completed,
    Staled,
}

#[derive(Serialize, Deserialize, Debug)]
struct Process {
    p_id: SUUID,
    name: Cow<'static, str>,
    status: OperataionStatus,
    create_at: u64,
    complete_at: u64,
    sla: u64,
}

pub fn to_u64(d: Duration) -> u64 {
    d.as_secs()
}

// Create a new Timestamp from a u64
pub fn from_u64(seconds: u64) -> Duration {
    Duration::new(seconds, 0)
}

pub fn to_string_time(t: u64) -> String {
    let dt = DateTime::from_timestamp(t as i64, 0).unwrap();
    let format = dt.format("%d-%m-%Y %H:%M:%S").to_string();
    format
}

#[tokio::main]
async fn main() -> Result<()> {
    let now = UNIX_EPOCH.elapsed().unwrap();
    println!("{:?}", to_u64(now));
    println!("{:?}", to_string_time(to_u64(now)));

    let database = db::new().await?;
    database.connect().await?;

    let _pp: Vec<Process> = database.conn
        .create("process")
        .content(Process {
            p_id: SUUID::new_v7(),
            name: "test".into(),
            status: OperataionStatus::New,
            create_at: to_u64(UNIX_EPOCH.elapsed().unwrap()),
            complete_at: 0,
            sla: 0, // TODO Default SLA FROM CONFIG
        })
        .await?;

    let process: Vec<Process> = database.conn.select("process").await?;

    println!("{:?}", process);

    Ok(())
}
