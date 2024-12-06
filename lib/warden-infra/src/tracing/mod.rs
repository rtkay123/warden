use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
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
}

impl Default for TelemetryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        let types: Box<dyn Layer<Registry> + Sync + Send> =
            tracing_subscriber::fmt::layer().boxed();
        TelemetryBuilder { layer: vec![types] }
    }

    /// Initialises tracing
    pub fn build(self) -> Telemetry {
        tracing_subscriber::registry()
            .with(self.layer)
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
            .init();
        Telemetry {}
    }
}
