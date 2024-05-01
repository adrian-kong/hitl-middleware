use crate::AppResult;
use chrono::{DateTime, Utc};
use lapin::{Connection, ConnectionProperties};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub http_client: Arc<Client>,
    pub db: PgPool,
    pub rmq: Arc<Connection>,
}

pub async fn setup_db() -> AppResult<PgPool> {
    let db_addr = std::env::var("DATABASE_URL")?;
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_addr)
        .await?)
}

pub async fn setup_rmq() -> AppResult<Connection> {
    let amqp_addr = std::env::var("AMQP_ADDR")?;
    Ok(Connection::connect(&amqp_addr, ConnectionProperties::default()).await?)
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    /// Waiting for inference
    Bot,
    /// Waiting for human feedback
    Human,
    /// Job succeeded
    Success,
    /// Job failed
    Fail,
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
