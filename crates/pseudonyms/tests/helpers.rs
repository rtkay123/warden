use sqlx::PgPool;
use tokio::sync::oneshot;
use tonic::transport::Channel;
use warden_core::pseudonyms::transaction_relationship::mutate_pseudonym_client::MutatePseudonymClient;
use warden_pseudonyms::state::{AppHandle, AppState, Services};
use warden_stack::{Configuration, cache::RedisManager};

use std::sync::Arc;

pub struct TestApp {
    state: AppHandle,
    pub mutate: MutatePseudonymClient<Channel>,
}

impl TestApp {
    pub async fn new(pool: PgPool) -> Self {
        let (tx, rx) = oneshot::channel();
        // Set port to 0 so tests can spawn multiple servers on OS assigned ports.
        //
        let config_path = "pseudonyms.toml";

        let config = config::Config::builder()
            .add_source(config::File::new(config_path, config::FileFormat::Toml))
            .build()
            .unwrap();

        let mut config = config.try_deserialize::<Configuration>().unwrap();
        config.application.port = 0;

        let cache = RedisManager::new(&config.cache).await.unwrap();

        let services = Services {
            postgres: pool,
            cache,
        };

        let state = AppHandle(Arc::new(AppState::new(services, config, None).unwrap()));

        dbg!(&state.addr.port());

        tokio::spawn(warden_pseudonyms::run(state.clone(), tx));
        let port = rx.await.expect("channel to be open");
        let addr = format!("http://[::1]:{port}");

        let mutation_client = MutatePseudonymClient::connect(addr.to_string())
            .await
            .expect("expect server to be running");

        Self {
            state,
            mutate: mutation_client,
        }
    }
}
