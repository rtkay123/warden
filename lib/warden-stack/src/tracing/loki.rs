use crate::Monitoring;

use super::TracingBuilder;
use super::tracing_builder::{IsUnset, SetLokiTask, State};
use tracing_subscriber::Layer;

impl<S: State> TracingBuilder<S> {
    pub fn loki(
        mut self,
        config: &crate::AppConfig,
        monitoring: &Monitoring,
    ) -> Result<TracingBuilder<SetLokiTask<S>>, crate::ServiceError>
    where
        S::LokiTask: IsUnset,
    {
        use std::str::FromStr;
        let url = FromStr::from_str(&monitoring.loki_endpoint.as_ref())
            .map_err(|_e| crate::ServiceError::Unknown)?;

        let (layer, task) = tracing_loki::builder()
            .label("service_name", config.name.as_ref())?
            .extra_field("pid", format!("{}", std::process::id()))?
            .build_url(url)?;

        self.layers.push(layer.boxed());

        Ok(self.loki_internal(task))
    }
}
