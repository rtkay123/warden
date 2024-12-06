pub mod rule_config;

use async_graphql::MergedObject;

#[derive(Default, Debug, MergedObject)]
pub struct Query(rule_config::RuleConfigQuery);
