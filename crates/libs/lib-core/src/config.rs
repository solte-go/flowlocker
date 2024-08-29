use std::sync::OnceLock;
use tracing::{error};

use lib_utils::env::{get_env, get_env_parse};

pub fn core_config() -> &'static CoreConfig {
    static INSTANCE: OnceLock<CoreConfig> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        CoreConfig::load_from_env().unwrap_or_else(|ex| {
            error!("{:<12} - config", "FATAL - WHILE LOADING CONF");
            panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}")
        })
    })
}

#[allow(non_snake_case)]
pub struct CoreConfig {
    // -- Db
    pub db_url: String,
    pub db_max_connection: i32,

    // -- Web
    pub web_folder: String,
}

impl CoreConfig {
    fn load_from_env() -> lib_utils::env::Result<CoreConfig> {
        let max_connection:i32 = get_env_parse("SERVICE_DB_MAX_CONN")?;

        Ok(CoreConfig {
            // -- Db
            db_url: get_env("SERVICE_DB_URL")?,
            db_max_connection: max_connection,

            // -- Web
            web_folder: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}
