pub mod error;
pub mod grpc_svc;
mod http_svc;
pub mod reload_stream;
mod version;

use grpc_svc::interceptor::MyInterceptor;
use http_svc::build_router;
use tonic::service::Routes;
use tower_http::trace::TraceLayer;
use warden_core::{
    FILE_DESCRIPTOR_SET,
    configuration::{
        routing::{
            mutate_routing_server::MutateRoutingServer, query_routing_server::QueryRoutingServer,
        },
        rule::{
            mutate_rule_configuration_server::MutateRuleConfigurationServer,
            query_rule_configuration_server::QueryRuleConfigurationServer,
        },
    },
};

use crate::{server::error::AppError, state::AppHandle};

pub fn serve(state: AppHandle) -> Result<(axum::Router, axum::Router), AppError> {
    let app = build_router(state.clone());

    let service = QueryRoutingServer::with_interceptor(state.clone(), MyInterceptor);

    let routing_reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let grpc_server = Routes::new(service)
        .add_service(MutateRoutingServer::with_interceptor(
            state.clone(),
            MyInterceptor,
        ))
        .add_service(MutateRuleConfigurationServer::with_interceptor(
            state.clone(),
            MyInterceptor,
        ))
        .add_service(QueryRuleConfigurationServer::with_interceptor(
            state.clone(),
            MyInterceptor,
        ))
        .add_service(routing_reflector)
        .into_axum_router()
        .layer(
            TraceLayer::new_for_grpc().make_span_with(|request: &axum::http::Request<_>| {
                tracing::trace_span!(env!("CARGO_PKG_NAME"), "otel.kind" = "server",
                    headers = ?request.headers()
                )
            }),
        );

    Ok((app, grpc_server))
}
