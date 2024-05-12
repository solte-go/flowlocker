mod db;
mod error;
mod model;
mod rest_api;
mod time;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use model::Process;
use std::time::{Duration, UNIX_EPOCH};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::sql::Uuid as SUUID;

pub use self::error::{Error, Result};

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use surrealdb::engine::local::Db;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

//use surrealdb::engine::local::Mem; uncomment after moving to in memory DB
//use once_cell::sync::Lazy;
//static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

#[tokio::main]
async fn main() -> Result<()> {
    //    let now = UNIX_EPOCH.elapsed().unwrap();
    //    println!("{:?}", to_u64(now));
    //    println!("{:?}", to_string_time(to_u64(now)));

    let database = db::new().await?;
    database.connect().await?;

    // let _pp: Vec<Process> = database
    //     .conn
    //     .create("process")
    //     .content(Process {
    //         p_id: SUUID::new_v7(),
    //         name: "test".into(),
    //         status: OperataionStatus::New,
    //         create_at: to_u64(UNIX_EPOCH.elapsed().unwrap()),
    //         complete_at: 0,
    //         sla: 0, // TODO Default SLA FROM CONFIG
    //     })
    //     .await?;

    let process: Vec<Process> = database.conn.select("process").await?;

    println!("{:?}", process);

    Ok(())
}
