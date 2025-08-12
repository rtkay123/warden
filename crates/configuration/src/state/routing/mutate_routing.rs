use opentelemetry_semantic_conventions::attribute;
use tonic::{Request, Response, Status, async_trait};
use tracing::{Instrument, error, info_span, instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;
use warden_core::configuration::{
    ReloadEvent,
    routing::{
        DeleteConfigurationRequest, RoutingConfiguration, UpdateRoutingRequest,
        mutate_routing_server::MutateRouting,
    },
};

use crate::state::{AppHandle, cache_key::CacheKey, invalidate_cache, publish_reload};

#[allow(dead_code)]
struct RoutingRow {
    id: Uuid,
    configuration: sqlx::types::Json<RoutingConfiguration>,
}

#[async_trait]
impl MutateRouting for AppHandle {
    #[instrument(skip(self, request))]
    async fn create_routing_configuration(
        &self,
        request: Request<RoutingConfiguration>,
    ) -> Result<Response<RoutingConfiguration>, Status> {
        let request = request.into_inner();
        let span = info_span!("create.configuration.routing");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "routing");

        sqlx::query!(
            "insert into routing (id, configuration) values ($1, $2)",
            Uuid::now_v7(),
            sqlx::types::Json(&request) as _
        )
        .execute(&self.services.postgres)
        .instrument(span)
        .await
        .map_err(|e| {
            error!("{e}");
            tonic::Status::internal(e.to_string())
        })?;

        Ok(tonic::Response::new(request))
    }

    async fn update_routing_configuration(
        &self,
        request: Request<UpdateRoutingRequest>,
    ) -> Result<Response<RoutingConfiguration>, Status> {
        let conf = self
            .app_config
            .nats
            .subject
            .split(".")
            .next()
            .expect("bad config");

        let request = request.into_inner();
        let id = Uuid::parse_str(&request.id)
            .map_err(|_e| tonic::Status::invalid_argument("id is not a uuid"))?;

        let config = request.configuration.expect("configuration to be provided");

        let span = info_span!("update.configuration.routing");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "update");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "routing");

        let updated = sqlx::query_as!(
            RoutingRow,
            r#"
                update routing
                set configuration = $1
                where id = $2
                returning id, configuration as "configuration: sqlx::types::Json<RoutingConfiguration>"
            "#,
            sqlx::types::Json(&config) as _,
            id
        )
        .fetch_one(&self.services.postgres)
        .instrument(span)
        .await
        .map_err(|e| {
            error!("{e}");
            tonic::Status::internal(e.to_string())
        })?;

        let (_del_result, _publish_result) = tokio::try_join!(
            invalidate_cache(self, CacheKey::Routing(&id)),
            publish_reload(self, conf, ReloadEvent::Routing)
        )?;

        let res = updated.configuration.0;

        Ok(Response::new(res))
    }

    async fn delete_routing_configuration(
        &self,
        request: Request<DeleteConfigurationRequest>,
    ) -> Result<Response<RoutingConfiguration>, Status> {
        let conf = self
            .app_config
            .nats
            .subject
            .split(".")
            .next()
            .expect("bad config");

        let request = request.into_inner();
        let id = Uuid::parse_str(&request.id)
            .map_err(|_e| tonic::Status::invalid_argument("id is not a uuid"))?;

        let span = info_span!("delete.configuration.routing");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "delete");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "routing");

        let updated = sqlx::query_as!(
            RoutingRow,
            r#"
                delete from routing
                where id = $1
                returning id, configuration as "configuration: sqlx::types::Json<RoutingConfiguration>"
            "#,
            id
        )
        .fetch_one(&self.services.postgres)
        .instrument(span)
        .await
        .map_err(|e| {
            error!("{e}");
            tonic::Status::internal(e.to_string())
        })?;

        let (_del_result, _publish_result) = tokio::try_join!(
            invalidate_cache(self, CacheKey::Routing(&id)),
            publish_reload(self, conf, ReloadEvent::Routing)
        )?;

        let res = updated.configuration.0;

        Ok(Response::new(res))
    }
}
