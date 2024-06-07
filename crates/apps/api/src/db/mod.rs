pub mod error;
pub mod repository;

use std::sync::Arc;
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
                username: "surreal_user",
                password: "dev_surreal_pass",
            })
            .await?;

        self.conn.use_ns("flowlocker").use_db("processes").await?;

        Ok(())
    }
}
