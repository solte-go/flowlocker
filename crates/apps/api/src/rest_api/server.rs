use axum::{middleware, Router};
//use signal_hook::iterator::Signals;
use tokio::net::TcpListener;
use tracing::info;
//use tokio::signal;
use crate::db::Database;
//use crate::shutdown_signal;

use super::error::Result;
use super::routes::routes;
use super::middleware::{mw_response_map, mw_ctx_resolver};

pub async fn new_server(db: Database) -> Result<()> {
    let routes_all = Router::new()
        .merge(routes(db.clone()))
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn(mw_ctx_resolver));

    //    let (close_tx, close_rx) = tokio::sync::oneshot::channel();

    info!("Starting axum server on port {}", 8050);

    let listener = TcpListener::bind("0.0.0.0:8050").await.unwrap();
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

//    tokio::spawn(async {
//        axum::serve(listener, routes_all.into_make_service())
//            .with_graceful_shutdown(async move {
//                _ = close_rx.await;
//            })
//            .await
//            .unwrap();
//    });

    Ok(())
}