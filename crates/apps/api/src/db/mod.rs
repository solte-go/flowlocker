pub mod error;

use std::sync::Arc;

use crate::config::config;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use self::error::Result;

#[derive(Clone, Debug)]
pub struct Database {
    pub conn: Arc<Surreal<Client>>,
}

pub async fn new() -> Result<Database> {
    Ok(Database {
        conn: Arc::new(Surreal::new::<Ws>("127.0.0.1:8000").await?),
    })
}

impl Database {
    pub async fn connect(&self) -> Result<()> {
        self.conn
            .signin(Root {
                username: config().db_user.as_str(),
                password: config().db_password.as_str(),
            })
            .await?;

        self.conn.use_ns("flowlocker").use_db("processes").await?;

        Ok(())
    }
}
