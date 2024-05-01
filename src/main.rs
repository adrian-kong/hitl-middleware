mod error;
mod model;
mod mq;

use crate::error::AppError;
use crate::model::{setup_db, setup_rmq, AppState, InferenceJobModel, JobStatus};
use crate::mq::handle_rmq;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use http_body_util::BodyExt;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub type AppResult<T> = Result<T, AppError>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let state = AppState {
        http_client: Arc::new(reqwest::Client::new()),
        db: setup_db().await.unwrap(),
        rmq: Arc::new(setup_rmq().await.unwrap()),
    };
    handle_rmq(&state);
    axum::serve(listener, setup_router(state)).await.unwrap();
}

pub fn setup_router(state: AppState) -> Router {
    Router::new()
        .route("/liveness", get(|| async { "ok" }))
        .route("/enqueue", post(enqueue_job))
        .route("/job", get(get_job_status))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn enqueue_job(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<impl IntoResponse, AppError> {
    let body = req.into_body().collect().await?.to_bytes().to_vec();
    let job_id = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>();
    sqlx::query!(
        "INSERT INTO inference_jobs(job_id, payload) VALUES($1, $2)",
        job_id,
        body
    )
    .execute(&state.db)
    .await?;
    Ok(Json(json!({"status": "success","job_id" : job_id})))
}

async fn get_job_status(
    State(state): State<AppState>,
    Query(id): Query<String>,
) -> Result<impl IntoResponse, AppError> {
    let model = sqlx::query_as!(
        InferenceJobModel,
        r#"SELECT job_id, status AS "status: JobStatus", payload, response, created_at FROM inference_jobs WHERE job_id = $1"#,
        id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(model))
}
