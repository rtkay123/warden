use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "RuleConfigInput")]
pub struct RuleConfig {
    pub id: String,
    pub cfg: String,
    pub description: String,
    pub config: Option<Metadata>,
    #[graphql(skip_input)]
    pub created_at: Option<OffsetDateTime>,
    #[graphql(skip_input)]
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "RuleConfigMetaData")]
pub struct Metadata {
    pub parameters: serde_json::Value,
    pub exit_conditions: Vec<ExitCondition>,
    pub bands: Vec<Band>,
}

#[derive(Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "RuleConfigExitCondition")]
pub struct ExitCondition {
    pub sub_rule_ref: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "RuleConfigBand")]
pub struct Band {
    pub sub_rule_ref: String,
    pub reason: String,
    pub lower_limit: Option<i64>,
    pub upper_limit: Option<i64>,
}
