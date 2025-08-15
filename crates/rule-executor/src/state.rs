use std::sync::Arc;

use async_nats::jetstream::Context;
use moka::future::Cache;
use tokio::sync::RwLock;
use tonic::transport::Endpoint;
use tracing::error;
use warden_core::configuration::{
    routing::{RoutingConfiguration, query_routing_client::QueryRoutingClient},
    rule::RuleConfigurationRequest,
};
use warden_stack::Configuration;

use crate::cnfg::LocalConfig;
use warden_middleware::grpc::interceptor::{Intercepted, MyInterceptor};

#[derive(Clone)]
pub struct Services {
    pub jetstream: Context,
}

pub type AppHandle = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub services: Services,
    pub local_cache: Arc<RwLock<Cache<RuleConfigurationRequest, RoutingConfiguration>>>,
    pub config: LocalConfig,
    pub query_routing_client: QueryRoutingClient<Intercepted>,
}

impl AppState {
    pub async fn new(services: Services, configuration: Configuration) -> anyhow::Result<Self> {
        let config: LocalConfig = serde_json::from_value(configuration.misc.clone())?;
        let channel = Endpoint::new(config.config_endpoint.to_string())?
            .connect()
            .await
            .inspect_err(|e| {
                error!(
                    endpoint = ?config.config_endpoint,
                    "could not connect to config service: {e}",
                )
            })?;

        let query_routing_client = QueryRoutingClient::with_interceptor(channel, MyInterceptor);

        Ok(Self {
            services,
            config,
            local_cache: Arc::new(RwLock::new(Cache::builder().build())),
            query_routing_client,
        })
    }
}
