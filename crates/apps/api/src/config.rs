use std::sync::OnceLock;
use std::time::Duration;

use tracing::{error};

use super::error::Result;

use lib_utils::env::{get_env, get_env_duration};

pub struct Config {
    pub development: String,

    // Scheduler
    pub sch_interval: Duration,

    // DB
    pub db_user: String,
    pub db_password: String,
}

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|ex| {
            error!("{:<12} - config", "FATAL - WHILE LOADING CONF");
            panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}")
        })
    })
}

impl Config {
    fn load_from_env() -> Result<Config> {
        let interval: Duration = get_env_duration("SCHEDULER_INTERVAL")?;

        let db_user = get_env("SURREAL_DB_USER")?;
        let db_password = get_env("SURREAL_DB_PASSWORD")?;

        let config = Config {
            development: get_env("DEVELOPMENT").unwrap_or_else(|_| "".to_string()),
            sch_interval: interval,
            db_user,
            db_password,
        };

        Ok(config)
    }
}
