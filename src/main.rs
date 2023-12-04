mod error;
mod store;
mod web;

use store::storage::Storage;
use std::time::Duration;
use std::borrow::Cow;
use std::net::SocketAddr;
use axum::{Router};
use axum::body::HttpBody;

use axum::response::{IntoResponse};

use crate::web::router::routes;

use serde::{Deserialize, Serialize};

pub use self::error::{Error, Result};


#[tokio::main]
async fn main() -> Result<()> {
    let store = Storage::init().await?;

    let state_routes = Router::new()
        .merge(routes(store.clone()));

    let router = Router::new()
        .merge(state_routes);



    let addr = SocketAddr::from(([0, 0, 0, 0], 8050));
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}


