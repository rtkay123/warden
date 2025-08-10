pub mod server;
pub mod state;

use std::sync::Arc;

use serde::Deserialize;
use state::AppHandle;
use tracing::{debug, trace};

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub something: Arc<str>,
}

pub async fn run(state: AppHandle, tx: tokio::sync::oneshot::Sender<u16>) -> anyhow::Result<()> {
    trace!("running migrations");
    sqlx::migrate!("./migrations")
        .run(&state.services.postgres)
        .await?;
    debug!("ran migrations");

    server::serve(state, tx).await
}
