mod cache_key;
mod routing;

use async_nats::jetstream::Context;
use sqlx::PgPool;
use std::{ops::Deref, sync::Arc};
use tonic::transport::Endpoint;
use tracing::error;
use warden_stack::{Configuration, cache::RedisManager};

use crate::{
    cnfg::LocalConfig,
    server::grpc::interceptor::{Intercepted, MyInterceptor},
};

#[derive(Clone)]
pub struct AppHandle(Arc<AppState>);

impl Deref for AppHandle {
    type Target = Arc<AppState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct Services {
    pub postgres: PgPool,
    pub cache: RedisManager,
    pub jetstream: Context,
}

pub struct AppState {
    pub services: Services,
    pub app_config: LocalConfig,
}

impl AppState {
    pub async fn create(
        services: Services,
        configuration: &Configuration,
    ) -> Result<AppHandle, AppError> {
        let local_config: LocalConfig = serde_json::from_value(configuration.misc.clone())?;

        Ok(AppHandle(Arc::new(Self {
            services,
            app_config: local_config,
        })))
    }
}
