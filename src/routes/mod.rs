mod external;
mod hitl;

use axum::Router;
use axum::routing::get;
use tower_http::trace::TraceLayer;

pub fn app_router() -> Router {
    Router::new().route("/liveness", get(|| async { "ok" }))
        .merge(external::routes())
        .merge(hitl::routes())
        .layer(TraceLayer::new_for_http())
}
