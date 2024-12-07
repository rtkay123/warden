pub mod entities;
pub mod server;
pub mod state;

use anyhow::Result;
use state::AppState;
use warden_infra::{Services, config::Configuration};

pub async fn run(services: Services, config: Configuration) -> Result<()> {
    let state = AppState { services, config };
    server::serve(state).await
}
