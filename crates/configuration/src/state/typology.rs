use warden_core::configuration::typology::{TypologyConfiguration, TypologyConfigurationRequest};

use crate::state::cache_key::CacheKey;

pub mod mutate_typology;
pub mod query_typology;

pub struct TypologyRow {
    pub configuration: sqlx::types::Json<TypologyConfiguration>,
}

impl<'a> From<&'a TypologyConfigurationRequest> for CacheKey<'a> {
    fn from(value: &'a TypologyConfigurationRequest) -> Self {
        Self::Typology {
            id: &value.id,
            version: &value.version,
        }
    }
}
