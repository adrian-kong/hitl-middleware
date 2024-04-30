use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub enum AppError {
    Reqwest(reqwest::Error),
    Axum(axum::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let (status, message) = match self {
            AppError::Reqwest(err) => {
                tracing::error!(%err, "error from reqwest lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to make request")
            }
            AppError::Axum(err) => {
                tracing::error!(%err, "error from axum lib");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
        };
        (status, message).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<axum::Error> for AppError {
    fn from(value: axum::Error) -> Self {
        Self::Axum(value)
    }
}
