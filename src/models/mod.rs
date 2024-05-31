use std::borrow::Cow;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub app: Cow<'static, str>,
    pub process_name: Cow<'static, str>,
    pub status: OperationStatus,
    pub create_at: u64,
    pub complete_at: u64,
    pub sla: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OperationStatus {
    New,
    InProgress,
    Completed,
    Staled,
}