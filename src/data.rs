use axum::body::Bytes;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use hyper::HeaderMap;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, ConnectionProperties};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::iter::Map;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier to check status
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub payload: Vec<u8>,
}

impl Job {
    pub fn new(payload: Vec<u8>) -> Self {
        let rng = rand::thread_rng();
        let id = rng
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        Self {
            id,
            created_at: Utc::now(),
            payload,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<reqwest::Client>,
    pub db: PgPool,
    pub rmq: Arc<lapin::Connection>,
}

const INFERENCE_QUEUE: &str = "inference-jobs";
const URL: &str = "https://example.com";

impl AppState {
    pub async fn new() -> Result<Self, lapin::Error> {
        let client = reqwest::Client::new();
        let db_addr = std::env::var("DATABASE_URL").unwrap();
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_addr)
            .await
            .unwrap();
        let amqp_addr = std::env::var("AMQP_ADDR").unwrap();
        let conn = lapin::Connection::connect(&amqp_addr, ConnectionProperties::default()).await?;
        Ok(Self {
            client: Arc::new(client),
            db,
            rmq: Arc::new(conn),
        })
    }

    /// Start listener for job processor
    pub fn start_processor(&self) {
        let conn = self.rmq.clone();
        let client = self.client.clone();
        let _db = self.db.clone();
        tokio::spawn(async move {
            let channel = conn.create_channel().await.unwrap();
            channel
                .queue_declare(
                    INFERENCE_QUEUE,
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();
            let mut consumer = channel
                .basic_consume(
                    INFERENCE_QUEUE,
                    "axum-consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();
            let db = _db;
            while let Some(delivery) = consumer.next().await {
                let delivery = delivery.expect("error in consumer");
                delivery.ack(BasicAckOptions::default()).await.expect("ack");
                let data = delivery.data;
                let job: Job = serde_cbor::from_slice(&data).unwrap();
                let request = client
                    .post(URL)
                    .header("Authorization", "Bearer ")
                    .body(job.payload)
                    .build()
                    .unwrap();
                let response = client.execute(request).await.unwrap();
                let bytes = response.bytes().await.unwrap();
                tracing::info!(?bytes);
                sqlx::query!(
                    r#"UPDATE inference_jobs
                    SET response = $1
                    WHERE job_id = $2"#,
                    bytes.to_vec(),
                    job.id
                )
                .execute(&db)
                .await
                .unwrap();
            }
        });
    }

    pub async fn publish_job(&self, job: Job) {
        let channel = self.rmq.create_channel().await.unwrap();
        channel
            .queue_declare(
                INFERENCE_QUEUE,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        let confirm = channel
            .basic_publish(
                "",
                INFERENCE_QUEUE,
                BasicPublishOptions::default(),
                &serde_cbor::to_vec(&job).unwrap(),
                BasicProperties::default(),
            )
            .await
            .unwrap()
            .await
            .unwrap();
    }
}
