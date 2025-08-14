#[cfg(any(feature = "nats-jetstream", feature = "nats-core"))]
pub mod nats {
    pub mod extractor {
        pub struct HeaderMap<'a>(pub &'a async_nats::HeaderMap);

        impl opentelemetry::propagation::Extractor for HeaderMap<'_> {
            fn get(&self, key: &str) -> Option<&str> {
                self.0
                    .get(async_nats::header::IntoHeaderName::into_header_name(key))
                    .map(|value| value.as_str())
            }

            fn keys(&self) -> Vec<&str> {
                self.0.iter().map(|(n, _v)| n.as_ref()).collect()
            }
        }
    }

    pub mod injector {
        pub struct HeaderMap<'a>(pub &'a mut async_nats::HeaderMap);

        impl opentelemetry::propagation::Injector for HeaderMap<'_> {
            fn set(&mut self, key: &str, value: String) {
                self.0.insert(key, value);
            }
        }
    }
}

#[cfg(feature = "opentelemetry-tonic")]
pub mod tonic {
    pub mod extractor {
        pub struct MetadataMap<'a>(pub &'a tonic::metadata::MetadataMap);
        impl opentelemetry::propagation::Extractor for MetadataMap<'_> {
            fn get(&self, key: &str) -> Option<&str> {
                self.0.get(key).and_then(|metadata| metadata.to_str().ok())
            }

            /// Collect all the keys from the MetadataMap.
            fn keys(&self) -> Vec<&str> {
                self.0
                    .keys()
                    .map(|key| match key {
                        tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                        tonic::metadata::KeyRef::Binary(v) => v.as_str(),
                    })
                    .collect::<Vec<_>>()
            }
        }
    }

    pub mod injector {
        pub struct MetadataMap<'a>(pub &'a mut tonic::metadata::MetadataMap);

        impl opentelemetry::propagation::Injector for MetadataMap<'_> {
            /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
            fn set(&mut self, key: &str, value: String) {
                if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes())
                    && let Ok(val) = tonic::metadata::MetadataValue::try_from(&value)
                {
                    self.0.insert(key, val);
                }
            }
        }
    }
}

use crate::Monitoring;

use super::TracingBuilder;
use super::tracing_builder::{IsUnset, SetOtelProvider, State};
use tracing_subscriber::Layer;

impl<S: State> TracingBuilder<S> {
    pub fn opentelemetry(
        mut self,
        config: &crate::AppConfig,
        monitoring: &Monitoring,
    ) -> Result<TracingBuilder<SetOtelProvider<S>>, crate::ServiceError>
    where
        S::OtelProvider: IsUnset,
    {
        use opentelemetry::{
            KeyValue,
            global::{self},
            trace::TracerProvider,
        };
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::{
            Resource,
            trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
        };
        use opentelemetry_semantic_conventions::{
            SCHEMA_URL,
            resource::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION},
        };
        use tracing_opentelemetry::OpenTelemetryLayer;

        global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let resource = Resource::builder()
            .with_schema_url(
                [
                    KeyValue::new(SERVICE_NAME, config.name.to_owned()),
                    KeyValue::new(SERVICE_VERSION, config.version.to_owned()),
                    KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, config.env.to_string()),
                ],
                SCHEMA_URL,
            )
            .with_service_name(config.name.to_string())
            .build();

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(monitoring.opentelemetry_endpoint.as_ref())
            .build()
            .unwrap();

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .with_id_generator(RandomIdGenerator::default())
            .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                1.0,
            ))))
            .build();

        global::set_tracer_provider(provider.clone());

        let layer = OpenTelemetryLayer::new(provider.tracer(config.name.as_ref().to_string()));
        self.layers.push(layer.boxed());

        Ok(self.otel_internal(provider))
    }
}
