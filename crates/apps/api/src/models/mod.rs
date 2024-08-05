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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_at: Option<DateTime<Utc>>,
    pub sla: u64,
}

impl Process {
    pub fn to_response(&self) -> ResponseProcess {
        let complete_at: Option<DateTime<Utc>> = if self.complete_at != 0 {
            Some(DateTime::from_timestamp(self.complete_at as i64, 0).unwrap_or_default())
        } else {
            None
        };

        ResponseProcess {
            process_id: self.process_id.clone(),
            app: self.app.clone(),
            process_name: self.process_name.clone(),
            status: self.status.clone(),
            create_at: DateTime::from_timestamp(self.create_at as i64, 0).unwrap_or_default(),
            updated_at: DateTime::from_timestamp(self.updated_at as i64, 0).unwrap_or_default(),
            complete_at,
            // complete_at: DateTime::from_timestamp(self.complete_at as i64, 0).unwrap_or_default(),
            sla: self.sla,
        }
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum OperationStatus {
    New,
    InProgress,
    Completed,
    Staled,
    Outdated,
}

// impl PartialEq<OperationStatus> for &OperationStatus {
//     fn eq(&self, other: &OperationStatus) -> bool {
//         todo!()
//     }
// }

impl OperationStatus {
    pub fn is_staled(&self) -> bool {
        if self.to_string() == OperationStatus::Staled.to_string() {
            return true;
        }
        false
    }

    pub fn is_completed(&self) -> bool {
        if self.to_string() == OperationStatus::Completed.to_string() {
            return true;
        }
        false
    }

    //TODO Change Staled to Outdated in whole app
    pub fn is_outdated(&self) -> bool {
        if self.to_string() == OperationStatus::Outdated.to_string() {
            return true;
        }
        false
    }
}

impl std::fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationStatus::New => write!(f, "New"),
            OperationStatus::Completed => write!(f, "Completed"),
            OperationStatus::InProgress => write!(f, "InProgress"),
            OperationStatus::Outdated => write!(f, "Outdated"),
            OperationStatus::Staled => write!(f, "Staled"),
        }
    }
}