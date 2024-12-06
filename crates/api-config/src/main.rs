use warden_infra::tracing::Telemetry;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _tracing = Telemetry::builder().build();
    api_config::run().await
}
