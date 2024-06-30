use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
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
            app: self.app.clone(),
            process_name: self.process_name.clone(),
            status: self.status.clone(),
            create_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.create_at as i64, 0), Utc),
            updated_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.updated_at as i64, 0), Utc),
            complete_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.complete_at as i64, 0), Utc),
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