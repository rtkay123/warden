use axum::{Router, http::Request};
use tower_http::trace::TraceLayer;
use tracing::info_span;

use super::REQUEST_ID_HEADER;

pub fn apply_trace_context_middleware<T: Clone + Send + Sync + 'static>(
    router: Router<T>,
) -> Router<T> {
    router.layer(
        TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
            let request_id = request
                .headers()
                .get(REQUEST_ID_HEADER)
                .expect("should have been applied already");

            info_span!(
                "http_request",
                request_id = ?request_id,
                headers = ?request.headers()
            )
        }),
    )
}
