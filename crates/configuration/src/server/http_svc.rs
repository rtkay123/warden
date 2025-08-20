mod routes;

use axum::{Router, response::IntoResponse};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
#[cfg(feature = "redoc")]
use utoipa_redoc::Servable;
#[cfg(feature = "scalar")]
use utoipa_scalar::Servable as _;

use crate::state::AppHandle;

const TAG_ROUTING: &str = "Routing";
const TAG_RULES: &str = "Rules";
const TAG_TYPOLOGIES: &str = "Typologies";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = TAG_ROUTING, description = "Operations related to routing configuration"),
    )
)]
pub struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get),
    path = "/",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
pub async fn health_check() -> impl IntoResponse {
    let name = env!("CARGO_PKG_NAME");
    let ver = env!("CARGO_PKG_VERSION");

    format!("{name} v{ver} is live")
}

pub fn build_router(state: AppHandle) -> Router {
    let (router, _api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(health_check))
        .nest("/api", routes::router(state))
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

    warden_middleware::apply(router)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use tower::ServiceExt;
    use warden_stack::cache::RedisManager;

    use crate::{
        server::http_svc::{build_router, routes::test_config},
        state::{AppState, Services},
    };

    #[sqlx::test]
    async fn health_check(pool: PgPool) {
        let config = test_config();

        let cache = RedisManager::new(&config.cache).await.unwrap();
        let client = async_nats::connect(&config.nats.hosts[0]).await.unwrap();
        let jetstream = async_nats::jetstream::new(client);

        let state = AppState::create(
            Services {
                postgres: pool,
                cache,
                jetstream,
            },
            &test_config(),
        )
        .await
        .unwrap();
        let app = build_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
