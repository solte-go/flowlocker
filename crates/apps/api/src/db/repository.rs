use std::borrow::Cow;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

use crate::db::Database;
use crate::models::{OperationStatus, Process};
use crate::time::{from_epoch, to_u64};

use lib_query_builder::builder::{Parameter, QueryBuilder, Conditions};

use tracing::{debug, info, instrument};
use uuid::{self, Uuid};

use super::error::{Error, Result};

#[derive(Serialize, Deserialize, Debug)]
struct UpdateProcess {
    status: OperationStatus,
    updated_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct UnlockProcess {
    status: OperationStatus,
    updated_at: u64,
    ended_at: u64,
}

pub async fn create_new_process(
    db: &Database,
    app_name: String,
    process: String,
    eta: u64,
) -> Result<String> {
    let new_process_id = Uuid::now_v7().to_string();

    let now_time = match UNIX_EPOCH.elapsed() {
        Ok(time) => to_u64(time),
        Err(_) => {
            return Err(Error::Repository(
                "System time is before the UNIX_EPOCH".to_string(),
            ))
        }
    };

    let _: Option<Process> = db
        .conn
        .create(("process", &new_process_id))
        .content(Process {
            process_id: new_process_id.clone().into(),
            process_name: process.into(),
            app: app_name.into(),
            status: OperationStatus::New,
            create_at: now_time,
            updated_at: now_time,
            ended_at: 0,
            sla: eta, // TODO Default SLA FROM CONFIG
        })
        .await?;

    Ok(new_process_id)
}

pub async fn update_process_status(db: &Database, id: &str, status: OperationStatus) -> Result<()> {
    match status {
        OperationStatus::Completed | OperationStatus::Canceled | OperationStatus::Outdated => {
            let _: Option<Process> = db
                .conn
                .update(("process", id))
                .merge(UnlockProcess {
                    status,
                    updated_at: from_epoch()?,
                    ended_at: from_epoch()?,
                })
                .await?;
        }
        _ => {
            let _: Option<Process> = db
                .conn
                .update(("process", id))
                .merge(UpdateProcess {
                    status,
                    updated_at: from_epoch()?,
                })
                .await?;
        }
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn get_process_by_id(db: &Database, id: &str) -> Result<Process> {
    let result: Option<Process> = db.conn.select(("process", id)).await?;

    info!("Get_process_by_id result: {:?}", result);

    if result.is_none() {
        return Err(Error::RecordNotFound);
    }

    Ok(result.unwrap())
}

#[instrument]
pub async fn get_running_processes(db: &Database) -> Result<Option<Vec<Process>>> {
    let mut response: surrealdb::Response = db
        .conn
        .query("SELECT * FROM type::table($table)")
        .bind(("table", "process"))
        .await?;

    let processes: Vec<Process> = response.take(0)?;
    if processes.is_empty() {
        return Ok(None);
    }

    Ok(Some(processes))
}

enum Column {
    App,
    ProcessName,
    Status,
}

impl Column {
    fn to_string(&self) -> Cow<'static, str> {
        match self {
            Column::App => Cow::Borrowed("app"),
            Column::ProcessName => Cow::Borrowed("process_name"),
            Column::Status => Cow::Borrowed("status"),
        }
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Column::App => "app".to_string(),
            Column::ProcessName => "process_name".to_string(),
            Column::Status => "status".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[instrument]
pub async fn get_processes(db: &Database, app: Option<String>, process_name: Option<String>, status: Option<OperationStatus>,
) -> Result<Option<Vec<Process>>> {
    let mut qb = QueryBuilder::default()
        .select("*")
        .from("type::table($table)", Parameter::StringArg("process".to_string()));

    if app.is_some() {
        qb = qb.filter(Column::App, Conditions::Eq, app.unwrap().to_string());
    }

    if process_name.is_some() {
        qb = qb.filter(Column::ProcessName, Conditions::Eq, process_name.unwrap().to_string());
    }

    if status.is_some() {
        qb = qb.and(Column::Status, Conditions::Eq, status.unwrap().to_string());
    }

    let (query, args) = qb.build().unwrap();

    let mut res = db.conn.query(query);

    for arg in args.iter() {
        res = res.bind((arg.0, arg.1));
    }

    let mut resp: surrealdb::Response = res.await?;

    let p: Vec<Process> = resp.take(0)?;

    Ok(Some(p))
}

#[instrument]
pub async fn check_running_processes(
    db: &Database,
    app: &str,
    process_name: &str,
) -> Result<Option<Vec<Process>>> {

    //TODO move to Tracing package
    // let tracer = get_global_trace("flowlocker".to_string());
    // tracer.start_with_context("check_running_processes", span_ctx);

    //TODO Create query separately for tracing and logging
    let mut response: surrealdb::Response = db.conn
        .query("SELECT * FROM type::table($table) WHERE app = $app AND process_name = $process_name AND status = $status")
        .bind(("table", "process"))
        .bind(("app", app))
        .bind(("process_name", process_name))
        .bind(("status", OperationStatus::New.to_string()))
        .await?;

    let processes: Vec<Process> = response.take(0)?;
    if processes.is_empty() {
        return Ok(None);
    }

    debug!("Get_process_by_id result: {:?}", processes);

    Ok(Some(processes))
}
#[instrument(skip(db))]
pub async fn delete_process_by_id(db: &Database, id: &str) -> Result<()> {
    let _: Option<Process> = db.conn.delete(("process", id)).await?;
    Ok(())
}
