use stack_up::{Configuration, Environment};
use std::{ops::Deref, sync::Arc};

use crate::{cnfg::LocalConfig, error::AppError};

#[derive(Clone)]
pub struct AppHandle(Arc<AppState>);

impl Deref for AppHandle {
    type Target = Arc<AppState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct Services {}

pub struct AppState {
    pub environment: Environment,
}

impl AppState {
    pub async fn create(configuration: &Configuration) -> Result<AppHandle, AppError> {
        let local_config: LocalConfig = serde_json::from_value(configuration.misc.clone())?;

        Ok(AppHandle(Arc::new(Self {
            environment: configuration.application.env,
        })))
    }
}
