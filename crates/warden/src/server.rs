mod routes;

use axum::{Router, routing::get};

use crate::state::AppHandle;

pub fn router(state: AppHandle) -> Router {
    Router::new().route("/", get(routes::health_check))
}

#[cfg(test)]
pub(crate) fn test_config() -> stack_up::Configuration {
    use stack_up::Configuration;

    let config_path = "warden.toml";

    let config = config::Config::builder()
        .add_source(config::File::new(config_path, config::FileFormat::Toml))
        .build()
        .unwrap();

    config.try_deserialize::<Configuration>().unwrap()
}
