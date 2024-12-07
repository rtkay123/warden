pub mod graphql;
use std::net::{Ipv6Addr, SocketAddr};

use anyhow::Result;
use async_graphql_axum::GraphQL;
use axum::{Router, response::IntoResponse, routing::get};
use graphql::ApiSchemaBuilder;
use tracing::info;

use crate::state::AppState;

pub async fn serve(state: AppState) -> Result<()> {
    let port = state.config.port;
    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
    let schema = ApiSchemaBuilder::build(state);

    let app = Router::new()
        .route("/", get(handle).post_service(GraphQL::new(schema)))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let socket_addr = listener.local_addr()?;

    info!(addr = ?socket_addr, "listening");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle() -> impl IntoResponse {
    #[cfg(feature = "playground")]
    return axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/"),
    ));

    #[cfg(not(feature = "playground"))]
    return health().await;
}

async fn health() -> impl IntoResponse {
    format!(
        "{} v{} is live",
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}
