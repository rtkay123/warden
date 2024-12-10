pub mod pacs002;
pub mod pacs008;
pub mod pain001;
pub mod pain013;

use anyhow::Result;
use std::{net::{Ipv6Addr, SocketAddr}, sync::Arc};
use tracing::info;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

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
    let port = state.config.port;
    let state = Arc::new(state);
    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));

    let (app, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        //        .nest("/api/pain001", pain001::router())
        //        .nest("/api/pain013", pain013::router())
        .nest("/api/pacs008", pacs008::router(state.clone()))
        .nest("/api/pacs002", pacs002::router(state))
        .split_for_parts();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = app.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    let socket_addr = listener.local_addr()?;

    info!(addr = ?socket_addr, "listening");

    axum::serve(listener, app).await?;

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
