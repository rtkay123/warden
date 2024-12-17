use anyhow::Result;
use async_nats::jetstream::Message;
use futures_util::{StreamExt, future::join_all};
use tracing::{debug, error};
use warden_infra::{Services, configuration::Configuration};

pub async fn listen(services: Services, config: Configuration) -> Result<()> {
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
            let deliver_subject = consumer.deliver_subject.as_ref().map(|v| {
                debug!("configuring push-based consumer = {}", consumer.name);
                v.to_string()
            });

            match deliver_subject {
                Some(deliver_subject) => {
                    let cons = async_nats::jetstream::consumer::push::Config {
                        durable_name,
                        deliver_subject,
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

        pull_consumers = Vec::with_capacity(pull_consumers.len());
        for (name, config) in pull_consumers_config.iter() {
            let consumer = stream.get_or_create_consumer(&name, config.clone()).await?;
            pull_consumers.push(consumer);
        }

        push_consumers = Vec::with_capacity(push_consumers.len());
        for (name, config) in push_consumers_config.iter() {
            let consumer = stream.get_or_create_consumer(&name, config.clone()).await?;
            push_consumers.push(consumer);
        }
    }

    let pushes = push_consumers
        .into_iter()
        .map(|consumer| tokio::spawn(handle_message(consumer)));

    let pulls = pull_consumers
        .into_iter()
        .map(|consumer| tokio::spawn(handle_messages(consumer)));

    let _ = tokio::join!(
        tokio::spawn(join_all(pushes)),
        tokio::spawn(join_all(pulls))
    );

    Ok(())
}

async fn handle_messages(
    consumer: async_nats::jetstream::consumer::Consumer<
        async_nats::jetstream::consumer::pull::Config,
    >,
) -> anyhow::Result<()> {
    let mut messages = consumer.messages().await?;
    debug!("consumer is ready to receive messages");

    while let Some(Ok(message)) = messages.next().await {
        process_message(message).await;
    }

    Ok(())
}

async fn process_message(message: Message) {
    let subject = message.subject.to_string();

    debug!(
        "subject: {subject} -- message: {:?}",
        std::str::from_utf8(&message.message.payload)
    );
    // acknowledge the message
    if let Err(e) = message.ack().await {
        error!("{e}");
    }
}

async fn handle_message(
    consumer: async_nats::jetstream::consumer::Consumer<
        async_nats::jetstream::consumer::push::Config,
    >,
) -> Result<()> {
    let mut messages = consumer.messages().await?;
    debug!("consumer is ready to receive messages");

    while let Some(Ok(message)) = messages.next().await {
        process_message(message).await;
    }

    Ok(())
}
