use crate::error::AppError;
use crate::model::{InferenceJobModel, JobStatus};
use crate::routes::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/job", get(get_status))
}

async fn get_status(
    State(state): State<AppState>,
    Query(id): Query<String>,
) -> Result<impl IntoResponse, AppError> {
    let model = sqlx::query_as!(
        InferenceJobModel,
        r#"SELECT job_id, status AS "status: JobStatus", payload, response, created_at
        FROM inference_jobs WHERE job_id = $1"#,
        id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(model))
}
