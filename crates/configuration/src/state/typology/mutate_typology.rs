use opentelemetry_semantic_conventions::attribute;
use tonic::{Request, Response, Status, async_trait};
use tracing::{Instrument, error, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;
use warden_core::configuration::{
    ConfigKind, ReloadEvent,
    typology::{
        DeleteTypologyConfigurationRequest, TypologyConfiguration, UpdateTypologyConfigRequest,
        mutate_typologies_server::MutateTypologies,
    },
};

use crate::state::{
    AppHandle, cache_key::CacheKey, invalidate_cache, publish_reload, typology::TypologyRow,
};

#[async_trait]
impl MutateTypologies for AppHandle {
    async fn create_typology_configuration(
        &self,
        request: Request<TypologyConfiguration>,
    ) -> Result<Response<TypologyConfiguration>, Status> {
        let request = request.into_inner();
        let span = info_span!("create.configuration.typology");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "typology");
        span.set_attribute("otel.kind", "client");

        sqlx::query!(
            "insert into typology (uuid, configuration) values ($1, $2)",
            Uuid::now_v7(),
            sqlx::types::Json(&request) as _,
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

    async fn update_typology_configuration(
        &self,
        request: Request<UpdateTypologyConfigRequest>,
    ) -> Result<Response<TypologyConfiguration>, Status> {
        let conf = self
            .app_config
            .nats
            .subject
            .split(".")
            .next()
            .expect("bad config");

        let request = request.into_inner();

        let config = request.configuration.expect("configuration to be provided");

        let span = info_span!("update.configuration.typology");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "update");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "typology");
        span.set_attribute("otel.kind", "client");

        sqlx::query!(
            r#"
                update typology
                set configuration = $1
                where id = $2 and version = $3
            "#,
            sqlx::types::Json(&config) as _,
            config.id,
            config.version,
        )
        .execute(&self.services.postgres)
        .instrument(span)
        .await
        .map_err(|e| {
            error!("{e}");
            tonic::Status::internal(e.to_string())
        })?;

        let (_del_result, _publish_result) = tokio::try_join!(
            invalidate_cache(
                self,
                CacheKey::Typology {
                    id: &config.id,
                    version: &config.version,
                }
            ),
            publish_reload(
                self,
                conf,
                ReloadEvent {
                    kind: ConfigKind::Typology.into(),
                    id: Some(config.id.to_owned()),
                    version: Some(config.version.to_owned()),
                }
            )
        )?;

        Ok(Response::new(config))
    }

    async fn delete_typology_configuration(
        &self,
        request: Request<DeleteTypologyConfigurationRequest>,
    ) -> Result<Response<TypologyConfiguration>, Status> {
        let conf = self
            .app_config
            .nats
            .subject
            .split(".")
            .next()
            .expect("bad config");

        let request = request.into_inner();

        let span = info_span!("delete.configuration.typology");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "delete");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "typology");
        span.set_attribute("otel.kind", "client");

        let updated = sqlx::query_as!(
            TypologyRow,
            r#"
                delete from typology
                where id = $1 and version = $2
                returning configuration as "configuration: sqlx::types::Json<TypologyConfiguration>"
            "#,
            request.id,
            request.version,
        )
        .fetch_one(&self.services.postgres)
        .instrument(span)
        .await
        .map_err(|e| {
            error!("{e}");
            tonic::Status::internal(e.to_string())
        })?;

        let (_del_result, _publish_result) = tokio::try_join!(
            invalidate_cache(
                self,
                CacheKey::Typology {
                    id: &request.id,
                    version: &request.version,
                }
            ),
            publish_reload(
                self,
                conf,
                ReloadEvent {
                    kind: ConfigKind::Typology.into(),
                    id: Some(request.id.to_owned()),
                    version: Some(request.version.to_owned()),
                }
            )
        )?;

        let res = updated.configuration.0;

        Ok(Response::new(res))
    }
}
