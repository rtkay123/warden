pub mod state;

use std::sync::Arc;

use anyhow::Result;
use async_nats::jetstream::Message;
use futures_util::StreamExt;
use opentelemetry::global;
use state::DatabaseClients;
use tracing::{Instrument, Span, debug, error, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_infra::{
    Services, configuration::Configuration, tracing::opentelemetry::NatsMetadataExtractor,
};

pub async fn listen(services: Services, config: Configuration) -> Result<()> {
    let transaction_history_client = services.postgres.try_into().unwrap();

    let clients = Arc::new(DatabaseClients {
        transaction_history: transaction_history_client,
    });

    let jetstream = services.jetstream.expect("nats not configured");
    let mut pull_consumers_config = vec![];
    let mut push_consumers_config = vec![];
    let mut pull_consumers = vec![];
    let mut push_consumers = vec![];

    for stream_config in config.nats.jetstream.iter() {
        let stream = if let Some(ref subjects) = stream_config.subjects {
            jetstream
                .get_or_create_stream(async_nats::jetstream::stream::Config {
                    name: stream_config.name.to_string(),
                    max_messages: stream_config.max_msgs,
                    subjects: subjects.iter().map(String::from).collect::<Vec<_>>(),
                    max_bytes: stream_config.max_bytes,
                    ..Default::default()
                })
                .await?
        } else {
            jetstream.get_stream(stream_config.name.to_string()).await?
        };

        for consumer in stream_config.consumers.as_ref() {
            let durable_name = consumer.durable.as_ref().map(|v| v.to_string());

            match consumer.deliver_subject {
                Some(ref sub) => {
                    debug!("configuring push-based consumer = {}", consumer.name);
                    let cons = async_nats::jetstream::consumer::push::Config {
                        durable_name,
                        deliver_subject: sub.to_string(),
                        deliver_group: consumer.deliver_group.as_ref().map(|f| f.to_string()),
                        ..Default::default()
                    };
                    push_consumers_config.push((consumer.name.clone(), cons));
                }
                None => {
                    let cons = async_nats::jetstream::consumer::pull::Config {
                        durable_name,
                        ..Default::default()
                    };
                    pull_consumers_config.push((consumer.name.clone(), cons))
                }
            }
        }

        for (name, config) in pull_consumers_config.iter() {
            let consumer = stream.get_or_create_consumer(&name, config.clone()).await?;
            pull_consumers.push(consumer);
        }

        for (name, config) in push_consumers_config.iter() {
            let consumer = stream.get_or_create_consumer(&name, config.clone()).await?;
            push_consumers.push(consumer);
        }
    }

    let mut pushes: Vec<_> = push_consumers
        .into_iter()
        .map(|consumer| tokio::spawn(handle_message(consumer, clients.clone()).in_current_span()))
        .collect();

    let pulls = pull_consumers
        .into_iter()
        .map(|consumer| tokio::spawn(handle_messages(consumer, clients.clone()).in_current_span()));

    pushes.extend(pulls);

    futures_util::future::join_all(pushes)
        .in_current_span()
        .await;

    Ok(())
}

async fn handle_messages(
    consumer: async_nats::jetstream::consumer::Consumer<
        async_nats::jetstream::consumer::pull::Config,
    >,
    client: Arc<DatabaseClients>,
) -> anyhow::Result<()> {
    let mut messages = consumer.messages().await?;
    debug!("pull consumer is ready to receive messages");

    while let Some(Ok(message)) = messages.next().await {
        process_message(message, Arc::clone(&client)).await;
    }

    Ok(())
}

async fn process_message(message: Message, client: Arc<DatabaseClients>) {
    let subject = message.subject.to_string();
    let span = info_span!("processing");

    message.headers.as_ref().map(|headers| {
        let parent_context = global::get_text_map_propagator(|propagator| {
            let extractor = NatsMetadataExtractor(headers);
            propagator.extract(&extractor)
        });
        dbg!(&headers);

        span.set_parent(parent_context);
    });
    let _ = span.enter();

    debug!("subject: {subject} -- message");
    tokio::time::sleep(std::time::Duration::from_millis(100))
        .instrument(info_span!("working"))
        .await;
    // acknowledge the message
    if let Err(e) = message.ack().await {
        error!("{e}");
    }
}

async fn handle_message(
    consumer: async_nats::jetstream::consumer::Consumer<
        async_nats::jetstream::consumer::push::Config,
    >,
    client: Arc<DatabaseClients>,
) -> Result<()> {
    let mut messages = consumer.messages().await?;
    debug!("push consumer is ready to receive messages");

    while let Some(Ok(message)) = messages.next().await {
        process_message(message, Arc::clone(&client)).await;
    }

    Ok(())
}
