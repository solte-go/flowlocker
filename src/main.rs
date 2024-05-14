mod db;
mod error;
mod model;
mod rest_api;
mod time;

use std::{time::Duration, sync::Arc};

use axum::Router;
use signal_hook::iterator::Signals;
use tokio::net::TcpListener;
use tokio::signal;

pub use self::error::{Error, Result};

//use surrealdb::engine::local::Mem; uncomment after moving to in memory DB
//use once_cell::sync::Lazy;
//static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

#[tokio::main]
async fn main() -> Result<()> {

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

    // let process: Vec<Process> = database.conn.select("process").await?;
//    println!("telling server to shutdown");
//    _ = close_tx.send(());


    // println!("{:?}", process);
    shutdown_signal(close_tx).await;

    Ok(())
}

async fn shutdown_signal(close_tx: tokio::sync::oneshot::Sender<()>) {
    let ctrl_c = async {
        _ = close_tx.send(());
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        println!("Exiting signal recieved")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}