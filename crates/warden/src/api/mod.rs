use anyhow::Result;
use axum::{extract::MatchedPath, http::Request};
use std::{
    net::{Ipv6Addr, SocketAddr},
    sync::Arc,
};
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

pub mod error;
pub mod pacs002;
pub mod pacs008;

const PACS008: &str = "pacs.008.001.12";
const PACS002: &str = "pacs.002.001.12";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = PACS008, description = "Submit a pacs.008.001.12 payload"),
        (name = PACS002, description = "Submit a pacs.002.001.12 payload"),
    )
)]
struct ApiDoc;

pub async fn serve(state: AppState) -> Result<()> {
    let port = state.config.application.port;
    let state = Arc::new(state);
    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));

    let (app, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        //        .nest("/api/pain001", pain001::router())
        //        .nest("/api/pain013", pain013::router())
        .nest("/api/pacs008", pacs008::router(state.clone()))
        .nest("/api/pacs002", pacs002::router(state))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .split_for_parts();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = app.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    let socket_addr = listener.local_addr()?;

    info!(addr = ?socket_addr, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    "ok"
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
