use sqlx::PgPool;
use std::{ops::Deref, sync::Arc};
use tonic::transport::Endpoint;
use tracing::error;
use warden_core::pseudonyms::transaction_relationship::mutate_pseudonym_client::MutatePseudonymClient;
use warden_stack::{Configuration, Environment, cache::RedisManager};

use crate::{
    cnfg::LocalConfig,
    error::AppError,
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
}

pub struct AppState {
    pub environment: Environment,
    pub mutate_pseudonym_client: MutatePseudonymClient<Intercepted>,
    pub services: Services,
    pub app_config: LocalConfig,
}

impl AppState {
    pub async fn create(
        services: Services,
        configuration: &Configuration,
    ) -> Result<AppHandle, AppError> {
        let local_config: LocalConfig = serde_json::from_value(configuration.misc.clone())?;

        let channel = Endpoint::new(local_config.pseudonyms_endpoint.to_string())?
            .connect()
            .await
            .inspect_err(|e| error!("could not connect to pseudonyms service: {e}"))?;

        let mutate_pseudonym_client =
            MutatePseudonymClient::with_interceptor(channel, MyInterceptor);

        Ok(AppHandle(Arc::new(Self {
            environment: configuration.application.env,
            mutate_pseudonym_client,
            services,
            app_config: local_config,
        })))
    }
}
