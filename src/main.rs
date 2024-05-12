mod db;
mod error;
mod model;
mod rest_api;
mod time;

use std::time::Duration;

use axum::Router;
use tokio::net::TcpListener;

pub use self::error::{Error, Result};

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

    let routes_all = Router::new()
        .merge(rest_api::routes(database));

    let (close_tx, close_rx) = tokio::sync::oneshot::channel();

    let listener = TcpListener::bind("0.0.0.0:8050").await.unwrap();

    // -- without threads --

//    axum::serve(listener, routes_all.into_make_service())
//        .await
//        .unwrap();

    let server_handle = tokio::spawn(async {
        axum::serve(listener, routes_all)
            .with_graceful_shutdown(async move {
                _ = close_rx.await;
            })
            .await
            .unwrap();
    });

    println!("waiting 5 seconds");
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("telling server to shutdown");
    _ = close_tx.send(());

    println!("waiting for server to gracefully shutdown");
    _ = server_handle.await;


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

    // let process: Vec<Process> = database.conn.select("process").await?;

    // println!("{:?}", process);

    Ok(())
}
