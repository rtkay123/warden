mod pacs002;
mod pacs008;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppHandle;

pub fn router(store: AppHandle) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(pacs008::post_pacs008))
        .routes(routes!(pacs002::post_pacs002))
        .with_state(store)
}
