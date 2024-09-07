mod config;
mod db;
mod error;
mod logger;
mod models;
mod repository;
mod rest_api;
mod scheduler;
mod time;

use tokio::signal;

use lib_core::tracing;

use config::config;

use tracing_subscriber::EnvFilter;

use log::{info, warn};
use tracing_error::ErrorLayer;

pub use self::error::{Error, Result};
use crate::scheduler::Scheduler;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// use surrealdb::engine::local::Mem; uncomment after moving to in memory DB
// use once_cell::sync::Lazy;
// static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

#[tokio::main]
async fn main() -> Result<()> {
    let tracer = tracing::init_opentelemetry("flowlocker".to_string())?;
    let subscriber = tracing_subscriber::fmt::layer().json();
    let tracer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(subscriber)
        .with(EnvFilter::from_default_env())
        .with(tracer)
        .with(ErrorLayer::default())
        .init();

    // FOR DEV ONLY
    if config().development == "dev" {
        warn!("LOADING DEVELOPMENT - development environment has been loaded");
        // All required dev setup goes here
    }

    let database = db::new().await?;
    database.connect().await?;

    let scheduler = Scheduler::new(database.clone(), config().sch_interval);

    scheduler.start().await?;

    let _run_axum = tokio::spawn(rest_api::server::new_server(database));

    info!("Listening for signals");

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

// TODO remove after refactoring
//
//
// let exporter = opentelemetry_stdout::LogExporterBuilder::default()
//     // uncomment the below lines to pretty print output.
//     // .with_encoder(|writer, data|
//     //     {
//     //         serde_json::to_writer_pretty(writer, &data).unwrap();
//     //         Ok(())
//     //     })
//     .build();
// let logger_provider = LoggerProvider::builder()
//     .with_config(config()
//         .with_resource(Resource::new(vec![KeyValue::new(
//             SERVICE_NAME,
//             "flowlocker",
//         )])))
//     .with_simple_exporter(exporter)
//     .build();

// Setup Log Appender for the log crate.
// let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
// log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
// log::set_max_level(Level::Debug.to_level_filter());

// let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
// let bunyan_formatting_layer =
//     BunyanFormattingLayer::new("flowlocker".to_string(), non_blocking_writer);
//
// let subscriber = Registry::default()
//     .with(EnvFilter::from_default_env())
//     .with(JsonStorageLayer);
//
// tracing::subscriber::set_global_default(subscriber).unwrap();
