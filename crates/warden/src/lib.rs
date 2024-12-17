pub mod api;
pub mod state;

use state::AppState;
use warden_infra::{Services, configuration::Configuration};

pub async fn serve(services: Services, configuration: Configuration) -> anyhow::Result<()> {
    let subs = serde_json::to_value(&configuration.nats.pub_subjects)?;
    let nats_subs = serde_json::from_value(subs)?;

    let state = AppState {
        cache: services.cache.expect("cache is none"),
        jetstream: services.jetstream.expect("jetstream is none"),
        config: configuration,
        nats_subjects: nats_subs,
    };

    api::serve(state).await
}
