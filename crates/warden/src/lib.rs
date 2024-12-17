pub mod api;
pub mod state;

use state::AppState;
use warden_infra::{Services, configuration::Configuration};

pub async fn serve(services: Services, configuration: Configuration) -> anyhow::Result<()> {
    let state = AppState {
        cache: services.cache.unwrap(),
        config: configuration,
    };

    api::serve(state).await
}
