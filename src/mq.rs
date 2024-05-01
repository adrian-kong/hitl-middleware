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
const URL: &str = "https://example.com";

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
            let job: InferenceJobModel = serde_cbor::from_slice(&delivery.data)?;
            if let Err(e) = on_job_received(&client, &db, job).await {
                tracing::error!(%e, "failed to process job");
            }
        }
        Ok::<(), AppError>(())
    });
}

async fn on_job_received(client: &Client, db: &PgPool, job: InferenceJobModel) -> AppResult<()> {
    let request = client
        .post(URL)
        .header("Authorization", "Bearer ")
        .body(job.payload)
        .build()?;
    let response = client.execute(request).await?;
    let bytes = response.bytes().await?;
    tracing::info!(?bytes);
    sqlx::query!(
        r#"UPDATE inference_jobs SET response = $1 WHERE job_id = $2"#,
        bytes.to_vec(),
        job.job_id
    )
    .execute(db)
    .await?;
    Ok(())
}

impl AppState {
    pub async fn publish_job(&self, job: InferenceJobModel) -> AppResult<Confirmation> {
        let confirmation = declare_inference_channel(&self.rmq)
            .await?
            .basic_publish(
                "",
                INFERENCE_QUEUE,
                BasicPublishOptions::default(),
                &serde_cbor::to_vec(&job)?,
                BasicProperties::default(),
            )
            .await?
            .await?;
        Ok(confirmation)
    }
}
