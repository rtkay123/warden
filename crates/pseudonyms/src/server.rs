mod interceptor;
use interceptor::MyInterceptor;
use tokio::signal;
use warden_core::pseudonyms::transaction_relationship::mutate_pseudonym_server::MutatePseudonymServer;

use tonic::transport::{Server, server::TcpIncoming};
use tracing::info;

use crate::state::AppHandle;

pub async fn serve(state: AppHandle, tx: tokio::sync::oneshot::Sender<u16>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(state.addr).await?;

    let socket_addr = listener
        .local_addr()
        .expect("should get socket_addr from listener");

    tx.send(socket_addr.port())
        .expect("port channel to be open");

    info!(addr = ?socket_addr, "starting server");

    Server::builder()
        .trace_fn(|_| tracing::info_span!(env!("CARGO_PKG_NAME")))
        //        .add_service(QueryUsersServer::new(state.clone()))
        .add_service(MutatePseudonymServer::with_interceptor(
            state.clone(),
            MyInterceptor,
        ))
        .serve_with_incoming_shutdown(TcpIncoming::from(listener), shutdown_signal(state))
        .await?;

    Ok(())
}
async fn shutdown_signal(state: AppHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            if let Some(ref provider) = state.tracer_provider {
                let _ = provider.shutdown();
            }
        },
        _ = terminate => {
            if let Some(ref provider) = state.tracer_provider {
                let _ = provider.shutdown();
            }
        },
    }
}
