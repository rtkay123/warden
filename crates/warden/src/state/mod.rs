use warden_infra::{Services, config::Configuration};

#[derive(Clone, Debug)]
pub struct AppState {
    pub services: Services,
    pub config: Configuration,
}
