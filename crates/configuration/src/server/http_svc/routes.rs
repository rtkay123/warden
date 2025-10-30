mod routing;
mod rule;
mod typology;

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
        .routes(routes!(
            /* typology */
            typology::get_typology::get_typology,
            typology::post_typology::update,
            typology::delete_typology::delete_typology,
            typology::create_typology::create_typology,
        ))
        .with_state(store)
}

#[cfg(test)]
pub(crate) fn test_config() -> warden_stack::Configuration {
    use warden_stack::Configuration;

    let config_path = "warden-config.toml";

    let config = config::Config::builder()
        .add_source(config::File::new(config_path, config::FileFormat::Toml))
        .build()
        .unwrap();

    config.try_deserialize::<Configuration>().unwrap()
}
