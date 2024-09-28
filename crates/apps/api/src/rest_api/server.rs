use axum::routing::get;
use axum::{middleware, Router};
//use signal_hook::iterator::Signals;
use tokio::net::TcpListener;
use tracing::info;
//use tokio::signal;
use crate::db::Database;
//use crate::shutdown_signal;
use lib_core::metrics::{
    metrics_handler, register_metrics, track_metrics, MetricsMiddleware, HTTP_REQUESTS_TOTAL,
    HTTP_REQUEST_DURATION,
};
use std::sync::Arc;

use super::error::Result;
use super::middleware::{log_result, mw_ctx_resolver, mw_response_map};
use super::routes::routes;

pub async fn new_server(db: Database) -> Result<()> {
    register_metrics();

    let metrics_middleware = Arc::new(MetricsMiddleware::new(
        HTTP_REQUESTS_TOTAL.clone(),
        HTTP_REQUEST_DURATION.clone(),
    ));

    let routes_all = Router::new()
        .merge(routes(db.clone()))
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn(log_result))
        .layer(middleware::from_fn(mw_ctx_resolver))
        .layer(middleware::from_fn(move |req, next| {
            let metrics = metrics_middleware.clone();
            track_metrics(metrics, req, next)
        }));

    info!("Starting axum server on port {}", 8050);

    let listener = TcpListener::bind("0.0.0.0:8050").await.unwrap();
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

pub async fn new_monitoring_server() -> Result<()> {
    let routes_all = Router::new().route("/metrics", get(metrics_handler));

    info!("Starting monitoring server on port {}", 2112);

    let listener = TcpListener::bind("0.0.0.0:2112").await.unwrap();
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}
