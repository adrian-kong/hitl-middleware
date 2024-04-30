use crate::error::AppError;
use axum::body::Body;
use axum::extract::FromRequest;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use axum::routing::{head, post};
use axum::{Extension, Router};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::upgrade::Upgraded;
use hyper::StatusCode;
use reqwest::Client;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpStream;
use tower_http::body::Full;

const URL: &str = "https://example.com";

pub fn routes() -> Router {
    let client = Arc::new(Client::new());
    Router::new()
        .route("/enqueue", post(enqueue))
        .layer(Extension(client))
}

async fn enqueue(
    Extension(client): Extension<Arc<Client>>,
    req: Request<Body>,
) -> Result<impl IntoResponse, AppError> {
    let mut headers = req.headers().clone();
    headers.remove("host");
    let body_bytes = req.into_body().collect().await?.to_bytes();
    let request = client.post(URL).headers(headers).body(body_bytes).build()?;
    let response = client.execute(request).await?;
    let content = response.text().await?;
    tracing::info!(?content);
    Ok(content)
}
