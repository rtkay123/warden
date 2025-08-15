use std::sync::Arc;

use anyhow::Result;
mod configuration;

use async_nats::jetstream::Message;
use warden_core::configuration::rule::RuleConfigurationRequest;

use crate::state::AppHandle;

pub async fn process_rule(message: Message, state: AppHandle) -> Result<()> {
    let req = create_configuration_request(&message);

    let rule_configuration = configuration::get_configuration(req, Arc::clone(&state)).await?;

    Ok(())
}

fn create_configuration_request(message: &Message) -> RuleConfigurationRequest {
    todo!()
}
