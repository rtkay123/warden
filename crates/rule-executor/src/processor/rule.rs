use std::sync::Arc;

use anyhow::Result;
mod configuration;

use async_nats::jetstream::Message;
use warden_core::configuration::rule::RuleConfigurationRequest;

use crate::state::AppHandle;

pub async fn process_rule(message: Message, state: AppHandle) -> Result<()> {
    let req = create_configuration_request(message.subject.as_str());

    let rule_configuration = configuration::get_configuration(req, Arc::clone(&state)).await?;

    Ok(())
}

fn create_configuration_request(subject: &str) -> RuleConfigurationRequest {
    // rule.901.v1.0.0
    let mut tokens = subject.split("rule.");
    // rule.
    tokens.next();
    // 901.v1.0.0
    let rem = tokens.next().expect("router guarantees subject");

    let mut tokens = rem.split(".v");
    let rule_id = tokens.next().expect("router guarantees subject");
    let version = tokens.next().expect("router guarantees subject");

    RuleConfigurationRequest {
        id: rule_id.to_owned(),
        version: version.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_subject() {
        let subject = "rule.901.v1.0.0";
        let req = create_configuration_request(subject);
        assert_eq!(req.id, "901");
        assert_eq!(req.version, "1.0.0");
    }

    #[test]
    fn test_valid_subject_with_longer_id() {
        let subject = "rule.12345.v2.3.4";
        let req = create_configuration_request(subject);
        assert_eq!(req.id, "12345");
        assert_eq!(req.version, "2.3.4");
    }

    #[test]
    #[should_panic(expected = "router guarantees subject")]
    fn test_missing_rule_prefix() {
        let subject = "901.v1.0.0"; // Missing "rule."
        create_configuration_request(subject);
    }

    #[test]
    #[should_panic(expected = "router guarantees subject")]
    fn test_missing_version() {
        let subject = "rule.901";
        create_configuration_request(subject);
    }

    #[test]
    fn test_different_version_format() {
        let subject = "rule.abc.v999";
        let req = create_configuration_request(subject);
        assert_eq!(req.id, "abc");
        assert_eq!(req.version, "999");
    }
}
