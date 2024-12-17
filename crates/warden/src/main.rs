use anyhow::Result;
use std::path::PathBuf;

use clap::Parser;
use warden_infra::{Services, configuration::Configuration};

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

    let services = Services::builder().with_cache(&config.cache).await?.build();
    println!("Hello, world!");

    Ok(())
}