use tracing::info;
use warden_infra::tracing::Telemetry;

#[tokio::main]
async fn main() {
    let _tracing = Telemetry::builder().build();
    info!("Hello, world!");
}
