#[cfg(feature = "opentelemetry")]
pub mod telemetry;

#[cfg(feature = "opentelemetry")]
pub use opentelemetry_sdk::trace::SdkTracerProvider;

#[cfg(feature = "tracing-loki")]
mod loki;

use tracing_subscriber::{
    EnvFilter, Layer, Registry, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Telemetry handle
#[derive(bon::Builder)]
#[builder(finish_fn(vis = "", name = build_internal))]
pub struct Tracing {
    #[builder(field = vec![tracing_subscriber::fmt::layer().boxed()])]
    layers: Vec<Box<dyn Layer<Registry> + Sync + Send>>,
    #[cfg(feature = "tracing-loki")]
    #[builder(setters(vis = "", name = loki_internal))]
    pub loki_task: tracing_loki::BackgroundTask,
    #[cfg(feature = "opentelemetry")]
    #[builder(setters(vis = "", name = otel_internal))]
    pub otel_provider: opentelemetry_sdk::trace::SdkTracerProvider,
}

// Define a custom finishing function as a method on the `UserBuilder`.
// The builder's state must implement the `IsComplete` trait.
// See details about it in the tip below this example.
impl<S: tracing_builder::IsComplete> TracingBuilder<S> {
    pub fn build(self, config: &crate::Monitoring) -> Tracing {
        // Delegate to `build_internal()` to get the instance of user.
        let mut tracing = self.build_internal();

        let layers = std::mem::take(&mut tracing.layers);
        tracing_subscriber::registry()
            .with(layers)
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| config.log_level.to_string().into()),
            )
            .try_init()
            .ok();
        tracing
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn build() {
//         let builder = Tracing::builder().build();
//         let level = crate::Monitoring {
//             log_level: "info".to_string(),
//             #[cfg(feature = "opentelemetry")]
//             opentelemetry_endpoint: "http://localhost:4317".into(),
//             #[cfg(feature = "tracing-loki")]
//             loki_endpoint: "http://localhost:3100".into(),
//         };
//         builder.init(&level);
//         builder.loki_task
//     }
// }
