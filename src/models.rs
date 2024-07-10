use crate::broker::Broker;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use strum_macros::{Display, EnumString};

#[derive(Deserialize, Serialize, EnumString, Debug, PartialEq, Display)]
pub enum SubmissionStatus {
    #[strum(serialize = "received")]
    #[serde(rename = "received")]
    Received,
    #[strum(serialize = "processing")]
    #[serde(rename = "processing")]
    Processing,
    #[strum(serialize = "completed")]
    #[serde(rename = "completed")]
    Completed,
    #[strum(serialize = "failed")]
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TaskSubmitRequest {
    pub task_name: String,
    pub args: Vec<Value>,
    pub kwargs: serde_json::Map<String, Value>,
}

#[derive(Deserialize, Serialize)]
pub struct TaskSubmitResponse {
    pub status: SubmissionStatus,
}

pub struct AppData {
    pub broker: Arc<Broker>,
}
