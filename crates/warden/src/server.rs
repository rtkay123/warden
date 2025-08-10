pub mod grpc;
mod middleware;
mod publish;
mod routes;

use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

#[cfg(feature = "redoc")]
use utoipa_redoc::Servable;
#[cfg(feature = "scalar")]
use utoipa_scalar::Servable as _;

use crate::{
    server::routes::{ApiDoc, metrics::metrics_app},
    state::AppHandle,
};

pub fn router(state: AppHandle) -> Router {
    let (router, _api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health_check))
        .nest("/api", routes::processor::router(state.clone()))
        .split_for_parts();

    #[cfg(feature = "swagger")]
    let router = router.merge(
        utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
            .url("/api-docs/swaggerdoc.json", _api.clone()),
    );

    #[cfg(feature = "redoc")]
    let router = router.merge(utoipa_redoc::Redoc::with_url("/redoc", _api.clone()));

    #[cfg(feature = "rapidoc")]
    let router = router.merge(
        utoipa_rapidoc::RapiDoc::with_openapi("/api-docs/rapidoc.json", _api.clone())
            .path("/rapidoc"),
    );

    #[cfg(feature = "scalar")]
    let router = router.merge(utoipa_scalar::Scalar::with_url("/scalar", _api));

    middleware::apply(router).merge(metrics_app())
}

/// Get health of the API.
#[utoipa::path(
    method(get),
    path = "/",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
pub async fn health_check() -> impl axum::response::IntoResponse {
    let name = env!("CARGO_PKG_NAME");
    let ver = env!("CARGO_PKG_VERSION");

    format!("{name} v{ver} is live")
}

#[cfg(test)]
pub(crate) fn test_config() -> warden_stack::Configuration {
    use warden_stack::Configuration;

    let config_path = "warden.toml";

    let config = config::Config::builder()
        .add_source(config::File::new(config_path, config::FileFormat::Toml))
        .build()
        .unwrap();

    config.try_deserialize::<Configuration>().unwrap()
}
