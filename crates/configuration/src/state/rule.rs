use uuid::Uuid;
use warden_core::configuration::rule::{RuleConfiguration, RuleConfigurationRequest};

use crate::state::cache_key::CacheKey;

mod mutate_rule;
mod query_rule;

#[allow(dead_code)]
pub struct RuleRow {
    pub uuid: Uuid,
    pub id: Option<String>,
    pub version: Option<String>,
    pub configuration: sqlx::types::Json<RuleConfiguration>,
}

impl<'a> From<&'a RuleConfigurationRequest> for CacheKey<'a> {
    fn from(value: &'a RuleConfigurationRequest) -> Self {
        Self::Rule {
            id: &value.id,
            version: &value.version,
        }
    }
}
