use opentelemetry_semantic_conventions::attribute;
use prost::Message;
use tonic::{Request, Status};
use tracing::{Instrument, debug, info_span, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_stack::redis::AsyncCommands;

use uuid::Uuid;
use warden_core::{
    configuration::routing::{
        GetActiveRoutingResponse, RoutingConfiguration, query_routing_server::QueryRouting,
    },
    google,
};

use crate::state::{AppHandle, cache_key::CacheKey};

pub struct RoutingRow {
    id: Uuid,
    configuration: sqlx::types::Json<RoutingConfiguration>,
}

#[tonic::async_trait]
impl QueryRouting for AppHandle {
    #[instrument(skip(self, _request), Err(Debug))]
    async fn get_active_routing_configuration(
        &self,
        _request: Request<google::protobuf::Empty>,
    ) -> Result<tonic::Response<GetActiveRoutingResponse>, Status> {
        let mut cache = self
            .services
            .cache
            .get()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let span = info_span!("cache.get");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
        span.set_attribute(attribute::DB_OPERATION_NAME, "get");
        span.set_attribute(attribute::DB_OPERATION_PARAMETER, "active");
        span.set_attribute("otel.kind", "client");

        let routing_config = cache
            .get::<_, Vec<u8>>(CacheKey::ActiveRouting)
            .instrument(span)
            .await
            .map(|value| {
                if !value.is_empty() {
                    RoutingConfiguration::decode(value.as_ref()).ok()
                } else {
                    None
                }
            });

        if let Ok(Some(routing_config)) = routing_config
            && routing_config.active
        {
            return Ok(tonic::Response::new(GetActiveRoutingResponse {
                configuration: Some(routing_config),
            }));
        }

        let span = info_span!("db.get.routing.active");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "select");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "routing");
        span.set_attribute(attribute::DB_OPERATION_PARAMETER, "active");
        span.set_attribute("otel.kind", "client");

        let config = sqlx::query_as!(
            RoutingRow,
            r#"select id, configuration as "configuration: sqlx::types::Json<RoutingConfiguration>" from routing where
            configuration->>'active' = 'true'"#,
        )
        .fetch_optional(&self.services.postgres)
        .instrument(span)
        .await.map_err(|e| tonic::Status::internal(e.to_string()))?;

        let config = config.map(|transaction| {
            debug!(id = ?transaction.id, "found active config");
            transaction.configuration.0
        });

        match config {
            Some(config) => {
                let bytes = config.encode_to_vec();
                let span = info_span!("cache.set");
                span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
                span.set_attribute(attribute::DB_OPERATION_NAME, "set");
                span.set_attribute(attribute::DB_OPERATION_PARAMETER, "routing.active");
                span.set_attribute("otel.kind", "client");

                if let Err(e) = cache
                    .set::<_, _, ()>(CacheKey::ActiveRouting, bytes)
                    .instrument(span)
                    .await
                {
                    warn!("{e}");
                };

                Ok(tonic::Response::new(GetActiveRoutingResponse {
                    configuration: Some(config),
                }))
            }
            None => Ok(tonic::Response::new(GetActiveRoutingResponse {
                configuration: None,
            })),
        }
    }
}
