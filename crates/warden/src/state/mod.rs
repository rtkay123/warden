use std::sync::Arc;

use async_nats::jetstream;
use serde::Deserialize;
use warden_infra::{cache::CacheService, configuration::Configuration};

pub struct AppState {
    pub cache: CacheService,
    pub jetstream: jetstream::Context,
    pub config: Configuration,
    pub nats_subjects: NatsSubjects
}

#[derive(Deserialize)]
pub struct NatsSubjects {
    pub transaction_history: Arc<str>,
    pub accounts: Arc<str>,
}
