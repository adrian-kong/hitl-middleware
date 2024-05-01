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
    // pub request_templates: HashMap<String, RequestTemplate>,
}
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct RequestTemplate {
//     pub url: String,
//     pub headers: HashMap<String, String>,
// }
//
// pub async fn setup_request_templates() -> AppResult<HashMap<String, RequestTemplate>> {
//     let content = tokio::fs::read_to_string("request_templates.json")
//         .await
//         .expect("Failed to read request_templates.json");
//     let templates = serde_json::from_str(&content).expect("Failed to parse request_templates.json");
//     Ok(templates)
// }

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
    pub payload: sqlx::types::JsonValue,
    pub response: Option<sqlx::types::JsonValue>,
    pub created_at: DateTime<Utc>,
}
