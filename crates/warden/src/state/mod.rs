use warden_infra::{cache::CacheService, configuration::Configuration};

pub struct AppState {
    pub cache: CacheService,
    pub config: Configuration,
}
