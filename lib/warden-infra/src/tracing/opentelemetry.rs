use opentelemetry::{global, trace::TraceContextExt};
use opentelemetry_http::HeaderExtractor;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::TelemetryBuilder;

impl TelemetryBuilder {
    #[cfg(feature = "opentelemetry")]
    /// Adds opentelemetry
    pub fn try_with_opentelemetry(
        mut self,
        config: &crate::configuration::App,
        metadata: &crate::configuration::Metadata,
    ) -> Result<Self, crate::ServiceError> {
        use opentelemetry::{KeyValue, global, trace::TracerProvider};
        use opentelemetry_otlp::{SpanExporter, WithExportConfig};
        use opentelemetry_sdk::{Resource, runtime};
        use opentelemetry_semantic_conventions::{
            SCHEMA_URL,
            resource::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION},
        };
        use tracing_opentelemetry::OpenTelemetryLayer;
        use tracing_subscriber::Layer;

        global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let resource = Resource::from_schema_url(
            [
                KeyValue::new(SERVICE_NAME, metadata.name.to_owned()),
                KeyValue::new(SERVICE_VERSION, metadata.version.to_owned()),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, config.environment.to_string()),
            ],
            SCHEMA_URL,
        );

        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.opentelemetry_endpoint)
            .build()
            .unwrap();

        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(exporter, runtime::Tokio)
            .with_resource(resource)
            .build();

        global::set_tracer_provider(provider.clone());
        let tracer = provider.tracer(metadata.name.to_string());

        self.layer.push(OpenTelemetryLayer::new(tracer).boxed());

        Ok(self)
    }
}

/// Attach trace headers to request
pub fn on_http_request(headers: &http::HeaderMap, span: &Span) {
    let parent_context =
        global::get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(headers)));
    span.set_parent(parent_context);
    let trace_id = span.context().span().span_context().trace_id();
    span.record("trace_id", trace_id.to_string());
}

#[cfg(feature = "nats-jetstream")]
use async_nats::{HeaderMap, HeaderName, HeaderValue};

use opentelemetry::propagation::{Extractor, Injector};

#[derive(Debug)]
/// Wrapper for [HeaderMap] implementing [Injector]
#[cfg(feature = "nats-jetstream")]
pub struct NatsMetadataInjector<'a>(pub &'a mut HeaderMap);

#[cfg(feature = "nats-jetstream")]
impl Injector for NatsMetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        let value = HeaderValue::from(value.as_str());
        self.0.insert(key, value);
    }
}

#[derive(Debug)]
#[cfg(feature = "nats-jetstream")]
/// Wrapper for [HeaderMap] implementing [Extractor]
pub struct NatsMetadataExtractor<'a>(pub &'a HeaderMap);

#[cfg(feature = "nats-jetstream")]
impl Extractor for NatsMetadataExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        <HeaderName as std::str::FromStr>::from_str(&key.to_lowercase())
            .ok()
            .and_then(|key| self.0.get(key).map(|value| value.as_str()))
    }

    /// Collect all the keys from the HashMap.
    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|(key, _value)| key.as_ref()).collect()
    }
}
