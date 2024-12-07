pub mod query;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use query::Query;

use crate::state::AppState;

pub struct ApiSchemaBuilder {}
pub type ApiSchema = Schema<Query, EmptyMutation, EmptySubscription>;

impl ApiSchemaBuilder {
    pub fn build(data: AppState) -> ApiSchema {
        Schema::build(Query::default(), EmptyMutation, EmptySubscription)
            .data(data)
            .finish()
    }
}
