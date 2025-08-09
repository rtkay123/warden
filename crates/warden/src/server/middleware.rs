mod metrics;
mod trace_layer;

pub use metrics::*;
pub use trace_layer::*;

use axum::{Router, http::HeaderName, middleware};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

use crate::server::middleware::apply_metrics_middleware;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn apply<T: Clone + Send + Sync + 'static>(router: Router<T>) -> Router<T> {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let router = router.layer(PropagateRequestIdLayer::new(x_request_id.clone()));

    apply_trace_context_middleware(router)
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
        .route_layer(middleware::from_fn(apply_metrics_middleware))
}
