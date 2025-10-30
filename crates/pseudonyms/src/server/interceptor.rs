use tonic::{Status, service::Interceptor};
use tracing::Span;
use warden_stack::{
    opentelemetry::global, tracing::telemetry::tonic::extractor,
    tracing_opentelemetry::OpenTelemetrySpanExt,
};

#[derive(Clone, Copy)]
pub struct MyInterceptor;

impl Interceptor for MyInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let span = Span::current();

        let cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&extractor::MetadataMap(request.metadata()))
        });

        if let Err(e) = span.set_parent(cx) {
            tracing::error!("{e:?}");
        };

        Ok(request)
    }
}
