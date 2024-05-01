mod data;
mod error;
mod routes;
mod model;

use crate::data::AppState;
use crate::routes::app_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let state = AppState::new().await.unwrap();
    state.start_processor();
    let router = app_router(state).await;
    axum::serve(listener, router).await.unwrap();
}
