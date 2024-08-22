use std::borrow::Cow;
use std::process;

use std::time::UNIX_EPOCH;
use opentelemetry::KeyValue;
use opentelemetry::Context;
use opentelemetry::trace::{Span, Status, Tracer};
use serde::{Deserialize, Serialize};

use surrealdb::sql::Uuid as SUUID;
use uuid::{self, Uuid};
use tracing::{info, instrument, span};
use lib_core::tracing::get_global_trace;

use crate::db::Database;
use crate::models::{OperationStatus, Process};
use crate::time::{to_u64, from_epoch};

use super::error::{Result, Error};

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

pub async fn create_new_process(db: &Database, app_name: String, process: String, eta: u64) -> Result<String> {
    let new_process_id = Uuid::now_v7().to_string();

    let now_time = match UNIX_EPOCH.elapsed() {
        Ok(time) => to_u64(time),
        Err(_) => return Err(Error::Repository("System time is before the UNIX_EPOCH".to_string())),
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
        }).await?;

    Ok(new_process_id)
}

pub async fn update_process_status(db: &Database, id: &str, status: OperationStatus) -> Result<()> {
    match status {
        OperationStatus::Completed | OperationStatus::Canceled | OperationStatus::Outdated => {
            let _: Option<Process> = db.conn.update(("process", id))
                .merge(UnlockProcess {
                    status,
                    updated_at: from_epoch()?,
                    ended_at: from_epoch()?,

                }).await?;
        }
        _ => {
            let _: Option<Process> = db.conn.update(("process", id))
                .merge(UpdateProcess {
                    status,
                    updated_at: from_epoch()?,
                }).await?;
        }
    }

    // if status == OperationStatus::Completed {
    //     let _: Option<Process> = db.conn.update(("process", id))
    //         .merge(UnlockProcess {
    //             status,
    //             updated_at: from_epoch()?,
    //             ended_at: from_epoch()?,
    //
    //         }).await?;
    // } else {
    //     let _: Option<Process> = db.conn.update(("process", id))
    //         .merge(UpdateProcess {
    //             status,
    //             updated_at: from_epoch()?,
    //         }).await?;
    // }

    Ok(())
}

#[instrument(skip(db))]
pub async fn get_process_by_id(db: &Database, id: &str) -> Result<Process> {
    // let p: Option<Process> = db.conn.select(("process", id)).await?;

    let result: Option<Process> = db.conn.select(("process", id)).await?;

    info!("Get_process_by_id result: {:?}", result);

    // let mut result: surrealdb::Response = db.conn
    //     .query("SELECT * FROM type::table($table) WHERE p_id = $id")
    //     .bind(("table", "process"))
    //     .bind(("id", "018fa928-0025-7ec3-9538-77d9c3ccf780"))
    //     .await?;

    // if let Err(e) = result.take::<Option<Process>>(0) {
    //         println!("Failed to retrieve a entry: {e:#?}");
    // }

    // let process: Option<Process> = result.take(0)?;

    if result.is_none() {
        return Err(Error::RecordNotFound);
    }

    Ok(result.unwrap())
}

#[instrument]
pub async fn get_running_processes(
    db: &Database,
) -> Result<Option<Vec<Process>>> {
    let mut response: surrealdb::Response = db.conn
        .query("SELECT * FROM type::table($table)")
        .bind(("table", "process"))
        .await?;


    let processes: Vec<Process> = response.take(0)?;
    if processes.is_empty() {
        return Ok(None);
    }

    // info!("Get_process_by_id result: {:?}", processes);

    Ok(Some(processes))
}

#[instrument]
pub async fn get_processes(
    db: &Database,
    app: Option<String>,
    process_name: Option<String>,
    status: Option<OperationStatus>
)->Result<Option<Vec<Process>>>
{

    let query = QueryBuilder::new()
        .select("*")
        .from("process")
        .filter("true")
        .if_then(app.is_some(), |q| {
            q.and("app".equals(&app.unwrap().quoted()))
        })
        .build();


    println!("query: {query}");

    Ok(None)
}



#[instrument]
pub async fn check_running_processes(
    db: &Database,
    app: &str,
    process_name: &str,
) -> Result<Option<Vec<Process>>> {
    println!("{:?}{:?}", app, process_name);


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

    info!("Get_process_by_id result: {:?}", processes);

    Ok(Some(processes))
}
#[instrument(skip(db))]
pub async fn delete_process_by_id(db: &Database, id: &str) -> Result<()> {
    let _: Option<Process> = db.conn.delete(("process", id)).await?;
    Ok(())
}
