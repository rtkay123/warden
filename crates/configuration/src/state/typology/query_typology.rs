use opentelemetry_semantic_conventions::attribute;
use prost::Message;
use tonic::{Request, Response, Status, async_trait};
use tracing::{Instrument, debug, info_span, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::configuration::typology::{
    GetTypologyConfigResponse, TypologyConfiguration, TypologyConfigurationRequest,
    query_typologies_server::QueryTypologies,
};
use warden_stack::redis::AsyncCommands;

use crate::state::{AppHandle, cache_key::CacheKey, typology::TypologyRow};

#[async_trait]
impl QueryTypologies for AppHandle {
    #[instrument(skip(self, request), Err(Debug))]
    async fn get_typology_configuration(
        &self,
        request: Request<TypologyConfigurationRequest>,
    ) -> Result<Response<GetTypologyConfigResponse>, Status> {
        let data = request.into_inner();
        let mut cache = self
            .services
            .cache
            .get()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let key = CacheKey::from(&data);

        let configuration = cache.get::<_, Vec<u8>>(&key).await.map(|value| {
            if !value.is_empty() {
                TypologyConfiguration::decode(value.as_ref()).ok()
            } else {
                None
            }
        });

        if let Ok(Some(typology_config)) = configuration {
            return Ok(tonic::Response::new(GetTypologyConfigResponse {
                configuration: Some(typology_config),
            }));
        }

        let span = info_span!("get.typology");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "select");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "typology");
        span.set_attribute("otel.kind", "client");

        let config = sqlx::query_as!(
            TypologyRow,
            r#"select configuration as "configuration: sqlx::types::Json<TypologyConfiguration>" from typology where
            id = $1 and version = $2"#,
            data.id,
            data.version,
        )
        .fetch_optional(&self.services.postgres)
        .instrument(span)
        .await.map_err(|e| tonic::Status::internal(e.to_string()))?;

        let config = config.map(|transaction| {
            debug!(id = ?transaction.configuration.0.id, "found config");
            transaction.configuration.0
        });

        match config {
            Some(config) => {
                let bytes = config.encode_to_vec();
                let span = info_span!("cache.set");
                span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
                span.set_attribute(attribute::DB_OPERATION_NAME, "set");
                span.set_attribute("otel.kind", "client");

                if let Err(e) = cache.set::<_, _, ()>(&key, bytes).instrument(span).await {
                    warn!("{e}");
                };

                Ok(tonic::Response::new(GetTypologyConfigResponse {
                    configuration: Some(config),
                }))
            }
            None => Ok(tonic::Response::new(GetTypologyConfigResponse {
                configuration: None,
            })),
        }
    }
}
