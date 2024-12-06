use async_graphql::{Context, Object};

#[derive(Default, Debug)]
pub struct RuleConfigQuery;

#[Object]
impl RuleConfigQuery {
    async fn get_rule_configs_by_id(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(String::default().into())
    }
}
