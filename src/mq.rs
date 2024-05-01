use crate::error::AppError;
use crate::model::{AppState, InferenceJobModel};
use crate::AppResult;
use futures_util::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions,
};
use lapin::publisher_confirm::Confirmation;
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection};
use reqwest::Client;
use sqlx::PgPool;

const INFERENCE_QUEUE: &str = "inference-jobs";

/// Just a helper method to declare "inference-jobs" channel in rabbitmq
async fn declare_inference_channel(conn: &Connection) -> Result<Channel, lapin::Error> {
    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            INFERENCE_QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    Ok(channel)
}

pub fn handle_rmq(state: &AppState) {
    let conn = state.rmq.clone();
    let client = state.http_client.clone();
    let db = state.db.clone();
    tokio::spawn(async move {
        let mut consumer = declare_inference_channel(&conn)
            .await?
            .basic_consume(
                INFERENCE_QUEUE,
                "axum-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
            let job_id = String::from_utf8(delivery.data).expect("should be utf8");
            let job = sqlx::query_as!(
                InferenceJobModel,
                r#"SELECT job_id, status AS "status: _", payload, response, created_at FROM inference_jobs WHERE job_id = $1"#,
                job_id
            )
            .fetch_one(&db)
            .await?;
            if let Err(e) = on_job_received(&client, &db, job).await {
                tracing::error!(%e, "failed to process job");
            }
        }
        Ok::<(), AppError>(())
    });
}

async fn on_job_received(client: &Client, db: &PgPool, job: InferenceJobModel) -> AppResult<()> {
    tracing::info!("processing job {}", job.job_id);
    let request = client
        .post("https://api.deepinfra.com/v1/openai/chat/completions")
        .header("Authorization", "Bearer ")
        .header("Content-Type", "application/json")
        .body(job.payload.to_string())
        .build()?;
    let response = client.execute(request).await?;
    let json = response.json::<serde_json::Value>().await?;
    tracing::info!("job {} returned with {}", job.job_id, json);
    sqlx::query!(
        r#"UPDATE inference_jobs SET response = $1, status = 'human' WHERE job_id = $2"#,
        json,
        job.job_id
    )
    .execute(db)
    .await?;
    Ok(())
}

impl AppState {
    pub async fn publish_job(&self, job_id: &str) -> AppResult<Confirmation> {
        let confirmation = declare_inference_channel(&self.rmq)
            .await?
            .basic_publish(
                "",
                INFERENCE_QUEUE,
                BasicPublishOptions::default(),
                job_id.as_bytes(),
                BasicProperties::default(),
            )
            .await?
            .await?;
        Ok(confirmation)
    }
}
