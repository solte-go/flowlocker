mod db;
mod error;
mod model;
mod rest_api;
mod time;
mod app_tracing;

use tokio::signal;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};


pub use self::error::{Error, Result};

//use surrealdb::engine::local::Mem; uncomment after moving to in memory DB
//use once_cell::sync::Lazy;
//static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

#[tokio::main]
async fn main() -> Result<()> {
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let bunyan_formatting_layer =
        BunyanFormattingLayer::new("flowlocker".to_string(), non_blocking_writer);

    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let _ = app_tracing::init_opentelemetry("flowlocker".to_string())?;

    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .json()
    //     .init();

    let database = db::new().await?;
    database.connect().await?;

    let _run_axum = tokio::spawn(rest_api::server::new_server(database));

    info!("Listening for signals");
    error!("Listening for signals");
    // println!("Listening for signals");
    shutdown_signal().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        println!("\nExiting signal received")
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