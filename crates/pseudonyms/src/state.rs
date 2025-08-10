mod mutate;

use std::{
    net::{Ipv6Addr, SocketAddr},
    ops::Deref,
    sync::Arc,
};

use sqlx::PgPool;
use warden_stack::{Configuration, cache::RedisManager, tracing::SdkTracerProvider};

use crate::AppConfig;

#[derive(Clone)]
pub struct AppHandle(pub Arc<AppState>);

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
}

#[derive(Clone)]
pub struct AppState {
    pub addr: SocketAddr,
    pub services: Services,
    pub config: Configuration,
    pub app_config: AppConfig,
    pub tracer_provider: Option<SdkTracerProvider>,
}

impl AppState {
    pub fn new(
        services: Services,
        config: Configuration,
        tracer_provider: Option<SdkTracerProvider>,
    ) -> anyhow::Result<Self> {
        let listen_address = SocketAddr::from((Ipv6Addr::UNSPECIFIED, config.application.port));

        let app_config: AppConfig = serde_json::from_value(config.misc.clone())?;

        Ok(Self {
            addr: listen_address,
            services,
            config,
            tracer_provider,
            app_config,
        })
    }
}
