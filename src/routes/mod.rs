mod external;
mod hitl;

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;
use crate::data::AppState;

pub async fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/liveness", get(|| async { "ok" }))
        .merge(external::routes())
        .merge(hitl::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
