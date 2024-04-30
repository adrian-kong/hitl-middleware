mod routes;
mod error;

use crate::routes::app_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app_router()).await.unwrap();
}

