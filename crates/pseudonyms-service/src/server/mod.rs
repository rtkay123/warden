use tonic::transport::Server;
use warden_core::pseudonyms::account_holder_service_server::AccountHolderServiceServer;

use crate::State;

pub async fn serve() -> anyhow::Result<()> {
    let addr = "[::1]:50051".parse()?;
    let greeter = State {};

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(AccountHolderServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
