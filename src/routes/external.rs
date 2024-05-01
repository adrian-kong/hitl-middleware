use crate::data::Job;
use crate::error::AppError;
use crate::routes::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Extension, Json, Router};
use http_body_util::BodyExt;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    let client = Arc::new(Client::new());
    Router::new()
        .route("/enqueue", post(enqueue))
        .layer(Extension(client))
}

async fn enqueue(
    State(state): State<AppState>,
    Extension(client): Extension<Arc<Client>>,
    req: Request<Body>,
) -> Result<impl IntoResponse, AppError> {
    let body = req.into_body().collect().await?.to_bytes().to_vec();
    let job = Job::new(body.clone());
    sqlx::query!(
        "INSERT INTO inference_jobs(job_id, payload) VALUES($1, $2)",
        job.id,
        body
    )
    .execute(&state.db)
    .await?;
    Ok(Json(json!({
        "status": "success",
        "job_id" : job.id
    })))
}
