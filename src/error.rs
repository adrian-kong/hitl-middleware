use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Lapin(#[from] lapin::Error),

    #[error(transparent)]
    SerdeCbor(#[from] serde_cbor::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Reqwest(err) => {
                tracing::error!(%err, "error from reqwest lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to make request")
            }
            AppError::Axum(err) => {
                tracing::error!(%err, "error from axum lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
            AppError::Sqlx(err) => {
                tracing::error!(%err, "error from sqlx lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
            AppError::Lapin(err) => {
                tracing::error!(%err, "error from lapin lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
            AppError::SerdeCbor(err) => {
                tracing::error!(%err, "error from cbor lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"),
        };
        (status, message).into_response()
    }
}
