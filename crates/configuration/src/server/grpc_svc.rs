pub mod interceptor {
    use opentelemetry::global;
    use tonic::{Status, service::Interceptor};
    use tracing::Span;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    use warden_stack::tracing::telemetry::tonic::extractor;

    #[derive(Clone, Copy)]
    pub struct MyInterceptor;

    impl Interceptor for MyInterceptor {
        fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
            let span = Span::current();

            let cx = global::get_text_map_propagator(|propagator| {
                propagator.extract(&extractor::MetadataMap(request.metadata()))
            });

            span.set_parent(cx);

            Ok(request)
        }
    }
}
