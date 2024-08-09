use serde::{Deserialize, Serialize};
use crate::models::OperationStatus;
use crate::rest_api::error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct GetProcess {}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct NewProcess {
    pub(crate) app: String,
    pub(crate) process: String,
    eta: String,
}

impl NewProcess {
    pub(crate) fn eta_to_u64(&self) -> crate::rest_api::error::Result<u64> {
        string_to_duration(self.eta.as_str())
    }
}

/// string_to_duration accept data in string format
/// #### Seconds
/// "60s"
/// #### Minutes
/// "5m"
/// #### Hours
/// "2h"
fn string_to_duration(duration: &str) -> crate::rest_api::error::Result<u64> {
    let len = duration.len();
    let (value, unit) = duration.split_at(len - 1);
    match value.parse::<u64>() {
        Ok(value) => match unit {
            "s" => Ok(value),
            "m" => Ok(value * 60),
            "h" => Ok(value * 60 * 60),
            _ => Err(ApiError::BadRequest("Invalid ETA format".to_string()))
        },
        Err(_) => Err(ApiError::BadRequest("Invalid ETA format".to_string())),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct UpdateProcess {
    status: OperationStatus,
}
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct UnlockProcess {}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestEndpoint {
    StartNewLock,
    GetLockedProcess,
}

impl ProcessData for UnlockProcess {
    fn get_status(&self) -> OperationStatus {
        OperationStatus::Completed
    }
}

impl ProcessData for UpdateProcess {
    fn get_status(&self) -> OperationStatus {
        self.status.clone()
    }
}

pub trait ProcessData {
    fn get_status(&self) -> OperationStatus;
}