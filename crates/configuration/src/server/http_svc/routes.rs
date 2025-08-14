mod routing;
mod rule;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppHandle;

pub fn router(store: AppHandle) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(
            /* routing */
            routing::get_active::active_routing,
            routing::post_routing::post_routing,
            routing::delete_routing::delete,
            routing::replace_routing::replace,
        ))
        .routes(routes!(
            /* rule */
            rule::create::create_rule,
            rule::update::update_rule_config,
            rule::delete::delete_rule_config,
            rule::get::get_rule,
        ))
        .with_state(store)
}
