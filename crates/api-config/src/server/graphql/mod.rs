pub mod query;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use query::Query;

use crate::handler::RuleHandler;

pub struct ApiSchemaBuilder {}
pub type ApiSchema = Schema<Query, EmptyMutation, EmptySubscription>;

impl ApiSchemaBuilder {
    pub fn build<T>(data: T) -> ApiSchema
    where
        T: RuleHandler + std::marker::Sync + std::marker::Send + 'static,
    {
        Schema::build(Query::default(), EmptyMutation, EmptySubscription)
            .data(data)
            .finish()
    }
}
