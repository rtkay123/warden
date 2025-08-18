use std::sync::Arc;

use anyhow::Result;
mod configuration;
mod determine_outcome;
mod rule_901;

use async_nats::jetstream;
use opentelemetry::global;
use tracing::{Span, debug, error, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{configuration::rule::RuleConfigurationRequest, message::Payload};
use warden_stack::tracing::telemetry::nats;

use crate::{processor::publish, state::AppHandle};

#[instrument(
    skip(message, state),
    err(Debug),
    fields(msg_id, rule_id, rule_version)
)]
pub async fn process_rule(message: jetstream::Message, state: AppHandle) -> Result<()> {
    let span = Span::current();

    if let Some(ref headers) = message.headers {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&nats::extractor::HeaderMap(headers))
        });
        span.set_parent(context);
    };

    let mut payload: Payload = prost::Message::decode(message.payload.as_ref())?;

    if payload.transaction.is_none() {
        warn!("transaction is empty - proceeding with ack");
        let _ = message
            .ack()
            .await
            .inspect_err(|e| warn!("ack failed: {e:?}"));
        return Ok(());
    }

    let transaction = payload
        .transaction
        .as_ref()
        .expect("none to have been handled");

    let msg_id = match transaction {
        warden_core::message::payload::Transaction::Pacs008(pacs008_document) => {
            &pacs008_document.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.msg_id
        }
        warden_core::message::payload::Transaction::Pacs002(pacs002_document) => {
            &pacs002_document.f_i_to_f_i_pmt_sts_rpt.grp_hdr.msg_id
        }
    };
    span.record("msg_id", msg_id);

    let req = create_configuration_request(message.subject.as_str());

    span.record("rule_id", &req.id);
    span.record("rule_version", &req.version);

    let config = configuration::get_configuration(req, Arc::clone(&state))
        .await
        .unwrap();

    match rule_901::process_901(&config, &payload, state.clone()).await {
        Ok(res) => {
            debug!(outcome = ?res.reason, "rule executed");
            payload.rule_result = Some(res);
            publish::to_typologies(&config.id, state, payload)
                .await
                .inspect_err(|e| error!("{e}"))?;
        }
        Err(e) => {
            error!("{e}");
        }
    };

    if let Err(e) = message.ack().await {
        error!("ack error {e:?}");
    };

    Ok(())
}

fn create_configuration_request(subject: &str) -> RuleConfigurationRequest {
    // rule.901.v1.0.0
    let mut tokens = subject.split("rule.");
    dbg!(&tokens);
    // rule.
    tokens.next();
    // 901.v1.0.0
    let rem = tokens.next().expect("router guarantees subject");

    let mut tokens = rem.split(".v");
    let rule_id = tokens.next().expect("router guarantees subject");
    let version = tokens.next().expect("router guarantees subject");

    RuleConfigurationRequest {
        id: rule_id.to_owned(),
        version: version.to_owned(),
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
