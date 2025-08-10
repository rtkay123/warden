warden-stack!

A crate centralising the configuration and initialisation of various services I find myself using in a lot of projects.

# TL:DR
Find a feature, activate it, initialise it in your builder, then you're good to go

## Features

Each feature unlocks configuration methods on the `ServicesBuilder`, allowing you to selectively wire up what you need.

| Feature        | Description                                               |
|----------------|-----------------------------------------------------------|
| `api`          | Enables `port` configuration     |
| `cache`        | Enables redis/valkey caching support   |
| `nats-core`    | Enables core NATS messaging via `async-nats`             |
| `nats-jetstream`| Enables NATS JetStream support via `async-nats`         |
| `opentelemetry`| Enables distributed tracing with OpenTelemetry           |
| `postgres`     | Enables PostgreSQL support using `sqlx`                  |
| `tracing`      | Enables tracing setup via `tracing` and `tracing-subscriber` |
| `opentelemetry-tonic`        | Enables opentelemetry injector and extractor utilities for `tonic`                    |
| `tracing-loki` | Enables tracing output to Loki.                           |
