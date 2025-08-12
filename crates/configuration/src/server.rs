mod error;
mod http_svc;
mod interceptor;
mod version;

use axum::http::header::CONTENT_TYPE;
use http_svc::build_router;
use interceptor::MyInterceptor;
use tonic::service::Routes;
use tower::{make::Shared, steer::Steer};
use warden_core::{
    FILE_DESCRIPTOR_SET,
    configuration::routing::{
        mutate_routing_server::MutateRoutingServer, query_routing_server::QueryRoutingServer,
    },
};

use crate::state::AppHandle;

pub async fn serve<S>(state: AppHandle) -> anyhow::Result<Shared<S>> {
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
        .add_service(routing_reflector)
        .into_axum_router();

    let service = Steer::new(
        vec![app, grpc_server],
        |req: &axum::extract::Request, _services: &[_]| {
            if req
                .headers()
                .get(CONTENT_TYPE)
                .map(|content_type| content_type.as_bytes())
                .filter(|content_type| content_type.starts_with(b"application/grpc"))
                .is_some()
            {
                // grpc service
                1
            } else {
                // http service
                0
            }
        },
    );

    Ok(Shared::new(service))
}
