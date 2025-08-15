pub mod interceptor {
    use tonic::{
        Status,
        service::{Interceptor, interceptor::InterceptedService},
        transport::Channel,
    };
    use tracing::Span;
    use warden_stack::{
        opentelemetry::global, tracing::telemetry::tonic::injector,
        tracing_opentelemetry::OpenTelemetrySpanExt,
    };

    pub type Intercepted = InterceptedService<Channel, MyInterceptor>;

    #[derive(Clone, Copy)]
    pub struct MyInterceptor;

    impl Interceptor for MyInterceptor {
        fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
            let cx = Span::current().context();
            global::get_text_map_propagator(|propagator| {
                propagator.inject_context(&cx, &mut injector::MetadataMap(request.metadata_mut()))
            });

            Ok(request)
        }
    }
}
