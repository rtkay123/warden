mod routing;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppHandle;

pub fn router(store: AppHandle) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(routing::get_active::active_routing))
        .with_state(store)
}
