use std::borrow::Cow;
use std::cmp::PartialEq;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub process_id: Cow<'static, str>,
    pub app: Cow<'static, str>,
    pub process_name: Cow<'static, str>,
    pub status: OperationStatus,
    pub create_at: u64,
    pub updated_at: u64,
    pub complete_at: u64,
    pub sla: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseProcess {
    pub process_id: Cow<'static, str>,
    pub app: Cow<'static, str>,
    pub process_name: Cow<'static, str>,
    pub status: OperationStatus,
    pub create_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub complete_at: DateTime<Utc>,
    pub sla: u64,
}

impl Process {
    pub fn to_response(&self) -> ResponseProcess {
        ResponseProcess {
            process_id: self.process_id.clone(),
            app: self.app.clone(),
            process_name: self.process_name.clone(),
            status: self.status.clone(),
            create_at: DateTime::from_timestamp(self.create_at as i64, 0).unwrap_or_default(),
            updated_at: DateTime::from_timestamp(self.updated_at as i64, 0).unwrap_or_default(),
            complete_at: DateTime::from_timestamp(self.complete_at as i64, 0).unwrap_or_default(),
            sla: self.sla,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationStatus {
    New,
    InProgress,
    Completed,
    Staled,
}

impl PartialEq<OperationStatus> for &OperationStatus {
    fn eq(&self, other: &OperationStatus) -> bool {
        todo!()
    }
}

impl OperationStatus {
    pub fn is_staled(&self) -> bool {
        if self.to_string() == OperationStatus::Staled.to_string() {
            return true;
        }
        false
    }
}

impl std::fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationStatus::New => write!(f, "New"),
            OperationStatus::InProgress => write!(f, "InProgress"),
            OperationStatus::Completed => write!(f, "Completed"),
            OperationStatus::Staled => write!(f, "Staled"),
        }
    }
}