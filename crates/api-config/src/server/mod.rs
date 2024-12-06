pub mod graphql;
use anyhow::Result;
use async_graphql_axum::GraphQL;
use axum::{Router, response::IntoResponse, routing::get};
use graphql::ApiSchemaBuilder;
use tokio::net::TcpListener;

use crate::state::AppState;

pub async fn serve() -> Result<()> {
    let schema = ApiSchemaBuilder::build(AppState::default());

    let app = Router::new()
        .route("/", get(handle).post_service(GraphQL::new(schema)))
        .route("/health", get(health));

    axum::serve(TcpListener::bind("127.0.0.1:8000").await?, app).await?;

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
