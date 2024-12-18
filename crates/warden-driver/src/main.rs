use anyhow::Result;
use std::path::PathBuf;

use clap::Parser;
use warden_infra::{
    Services,
    configuration::{Configuration, Metadata},
    tracing::TelemetryBuilder,
};

/// Warden API
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::Config::builder()
        .add_source(config::File::new(
            args.config_file
                .to_str()
                .expect("config file path is not valid"),
            config::FileFormat::Toml,
        ))
        .build()?;
    let config = config.try_deserialize::<Configuration>()?;
    dbg!(&config);

    let metadata = Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };

    let _tracing = TelemetryBuilder::new(config.application.log_level)
        .try_with_opentelemetry(&config.application, &metadata)?
        .build();

    let services = Services::builder()
        .with_nats_jetstream(&config.nats)
        .await?
        .with_cache(&config.cache)
        .await?
        .with_postgres(&config.database)
        .await?
        .build();

    warden_driver::listen(services, config).await
}
