use sqlx::PgPool;
use std::{ops::Deref, sync::Arc};

use async_nats::jetstream::Context;
use warden_stack::{Configuration, cache::RedisManager};

use crate::cnfg::LocalConfig;

#[derive(Clone)]
pub struct Services {
    pub jetstream: Context,
    pub cache: RedisManager,
    pub postgres: PgPool,
}

#[derive(Clone)]
pub struct AppState {
    pub services: Services,
    pub config: LocalConfig,
}

#[derive(Clone)]
pub struct AppHandle(pub Arc<AppState>);

impl Deref for AppHandle {
    type Target = Arc<AppState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub async fn create(
        services: Services,
        configuration: &Configuration,
    ) -> anyhow::Result<AppHandle> {
        let config = serde_json::from_value(configuration.misc.clone())?;

        Ok(AppHandle(Arc::new(Self { services, config })))
    }
}
