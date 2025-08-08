pub mod processor;

use utoipa::OpenApi;

const PACS008_001_12: &str = "pacs.008.001.12";
const PACS002_001_12: &str = "pacs.002.001.12";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = PACS008_001_12, description = "Submit a pacs.008.001.12 payload"),
        (name = PACS002_001_12, description = "Submit a pacs.002.001.12 payload"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{
        server::{self, test_config},
        state::AppState,
    };

    #[tokio::test]
    async fn health_check() {
        let state = AppState::create(&test_config()).await.unwrap();
        let app = server::router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
