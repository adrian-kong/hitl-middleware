use chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Success,
    Error,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct InferenceJobModel {
    pub job_id: String,
    pub status: JobStatus,
    pub payload: Vec<u8>,
    pub response: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}
