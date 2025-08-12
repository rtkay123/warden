use async_nats::jetstream::Context;
use tracing::{debug, info};

use crate::cnfg::JetstreamConfig;

pub async fn create_stream(jetstream: &Context, config: &JetstreamConfig) -> anyhow::Result<()> {
    debug!(name = ?config.stream, "initialising stream");

    jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: config.stream.to_string(),
            max_messages: config.max_messages,
            subjects: vec![config.subject.to_string()],
            ..Default::default()
        })
        .await?;

    info!(name = ?config.stream, subject = ?config.subject, "stream is ready");

    Ok(())
}
