use std::sync::Arc;
use surrealdb::sql::Value;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;
pub use crate::{Error, Result};

#[derive(Clone)]
pub struct Storage{
    pub ds: Arc<Surreal<Db>>
}

impl Storage {
    pub async fn init() -> Result<Storage> {
        let ds = Arc::new(Surreal::new::<Mem>(()).with_capacity(100_00).await?);
        let _ses = ds.use_ns("flowlocker").use_db("ps_sync").await?;

        Ok(Storage{ds})
    }
}