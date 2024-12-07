use async_graphql::{Context, Object};

use crate::entities::rule_config::RuleConfig;

#[derive(Default, Debug)]
pub struct RuleConfigQuery;

#[Object]
impl RuleConfigQuery {
    async fn get_rule_configs_by_id(
        &self,
        ctx: &Context<'_>,
        cfg: Option<String>,
    ) -> async_graphql::Result<Vec<RuleConfig>> {
        Ok(Default::default())
    }
}
