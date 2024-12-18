#[cfg(feature = "opentelemetry")]
/// Opentelemetry
pub mod opentelemetry;

use crate::configuration::LogLevel;
use tracing_subscriber::{
    EnvFilter, Layer, Registry, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Telemetry handle
#[allow(missing_debug_implementations)]
pub struct Telemetry {}

impl Telemetry {
    /// Create a new builder
    pub fn builder() -> TelemetryBuilder {
        TelemetryBuilder::default()
    }
}

/// A builder for initialising [tracing] layers
#[allow(missing_debug_implementations)]
pub struct TelemetryBuilder {
    layer: Vec<Box<dyn Layer<Registry> + Sync + Send>>,
    log_level: LogLevel,
}

impl Default for TelemetryBuilder {
    fn default() -> Self {
        Self::new(LogLevel::default())
    }
}

impl TelemetryBuilder {
    /// Create a new builder
    pub fn new(log_level: LogLevel) -> Self {
        let types: Box<dyn Layer<Registry> + Sync + Send> =
            tracing_subscriber::fmt::layer().boxed();
        TelemetryBuilder {
            layer: vec![types],
            log_level,
        }
    }

    /// Initialises tracing
    pub fn build(self) -> Telemetry {
        tracing_subscriber::registry()
            .with(self.layer)
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| self.log_level.into()))
            .init();
        Telemetry {}
    }
}
