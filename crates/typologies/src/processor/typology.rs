use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use opentelemetry::global;
use prost::Message;
use tracing::{Instrument, Span, error, info, info_span, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{
    configuration::{
        routing::RoutingConfiguration,
        typology::{TypologyConfiguration, TypologyConfigurationRequest},
    },
    message::{Payload, RuleResult, TypologyResult},
};
use warden_stack::{redis::AsyncCommands, tracing::telemetry::nats::extractor};

use crate::{
    processor::{driver::GetTypologyConfiguration as _, publish},
    state::AppHandle,
};

#[instrument(skip(message, state), err(Debug))]
pub async fn process_typology(
    message: async_nats::jetstream::Message,
    state: AppHandle,
) -> Result<()> {
    let span = Span::current();

    if let Some(ref headers) = message.headers {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&extractor::HeaderMap(headers))
        });
        span.set_parent(context);
    };

    let payload: Payload = Message::decode(message.payload.as_ref())?;

    if payload.transaction.is_none() {
        warn!("transaction is empty - proceeding with ack");
        let _ = message.ack().await;
        return Ok(());
    }

    let transaction = payload.transaction.as_ref().expect("to have returned");

    match transaction {
        warden_core::message::payload::Transaction::Pacs008(_) => {
            warn!("Pacs008 is unsupported on this version: this should be unreachable");
        }
        warden_core::message::payload::Transaction::Pacs002(pacs002_document) => {
            let key = format!(
                "tp_{}",
                pacs002_document.f_i_to_f_i_pmt_sts_rpt.grp_hdr.msg_id
            );

            let rule_result = &payload
                .rule_result
                .as_ref()
                .expect("rule result should be here");
            let rule_results = cache_and_get_all(&key, rule_result, Arc::clone(&state)).await?;

            let routing = payload
                .routing
                .as_ref()
                .expect("routing missing from payload");

            let (mut typology_result, _rule_count) =
                aggregate_rules(&rule_results, routing, rule_result)?;

            let _ = evaluate_typology(&mut typology_result, routing, payload.clone(), &key, state)
                .await
                .inspect_err(|e| error!("{e}"));
        }
    };

    let span = info_span!("nats.ack");
    message
        .ack()
        .instrument(span)
        .await
        .map_err(|_| anyhow::anyhow!("ack error"))?;

    Ok(())
}

#[instrument(skip(typology_result, routing, payload, state), err(Debug))]
async fn evaluate_typology(
    typology_result: &mut [TypologyResult],
    routing: &RoutingConfiguration,
    mut payload: Payload,
    key: &str,
    state: AppHandle,
) -> Result<()> {
    for typology_result in typology_result.iter_mut() {
        let handle = Arc::clone(&state);
        let routing_rules = routing.messages[0].typologies.iter().find(|typology| {
            typology.version.eq(&typology_result.version) && typology.id.eq(&typology_result.id)
        });
        let typology_result_rules = &typology_result.rule_results;

        if routing_rules.is_some()
            && typology_result_rules.len() < routing_rules.unwrap().rules.len()
        {
            continue;
        }

        let typology_config = handle
            .get_typology_config(TypologyConfigurationRequest {
                id: typology_result.id.to_owned(),
                version: typology_result.version.to_owned(),
            })
            .await?;

        let result = evaluate_expression(typology_result, &typology_config)?;

        typology_result.result = result;

        let workflow = typology_config
            .workflow
            .as_ref()
            .expect("no workflow in config");

        if workflow.interdiction_threshold.is_some() {
            typology_result.workflow.replace(*workflow);
        }
        typology_result.review = result.ge(&typology_config.workflow.unwrap().alert_threshold);

        payload.typology_result = Some(typology_result.to_owned());

        let is_interdicting = typology_config
            .workflow
            .unwrap()
            .interdiction_threshold
            .is_some_and(|value| value > 0.0 && result >= value);

        if is_interdicting {
            typology_result.review = true;
        }

        if result >= typology_config.workflow.unwrap().alert_threshold {
            info!("alerting");
        }

        let subj = handle.config.nats.destination_prefix.to_string();
        let _ = publish::to_tadp(&subj, handle, payload.clone())
            .await
            .inspect_err(|e| error!("{e}"));

        let mut c = state.services.cache.get().await?;
        c.del::<_, ()>(key).await?;
    }

    Ok(())
}

fn evaluate_expression(
    typology_result: &mut TypologyResult,
    typology_config: &TypologyConfiguration,
) -> Result<f64> {
    let mut to_return = 0.0;
    let expression = typology_config
        .expression
        .as_ref()
        .expect("expression is missing");

    let rule_values = &typology_config.rules;

    for rule in expression.terms.iter() {
        let rule_result = typology_result
            .rule_results
            .iter()
            .find(|value| value.id.eq(&rule.id) && value.version.eq(&rule.version));

        if rule_result.is_none() {
            warn!(term = ?rule, "could not find rule result for typology term");
            return Ok(Default::default());
        }

        let rule_result = rule_result.expect("checked and is some");

        let weight = rule_values
            .iter()
            .filter_map(|rv| {
                if !(rv.id.eq(&rule_result.id) && rv.version.eq(&rule_result.version)) {
                    None
                } else {
                    rv.wghts.iter().find_map(|value| {
                        match value.r#ref.eq(&rule_result.sub_rule_ref) {
                            true => Some(value.wght),
                            false => None,
                        }
                    })
                }
            })
            .next();

        if weight.is_none() {
            warn!(rule = ?rule, "could not find a weight for the matching rule");
        }
        let weight = weight.unwrap_or_default();

        to_return = match expression.operator() {
            warden_core::configuration::typology::Operator::Add => to_return + weight,
            warden_core::configuration::typology::Operator::Multiply => to_return * weight,
            warden_core::configuration::typology::Operator::Subtract => to_return - weight,
            warden_core::configuration::typology::Operator::Divide => {
                if weight.ne(&0.0) {
                    to_return / weight
                } else {
                    to_return
                }
            }
        };
    }
    Ok(to_return)
}

async fn cache_and_get_all(
    cache_key: &str,
    rule_result: &RuleResult,
    state: AppHandle,
) -> Result<Vec<RuleResult>> {
    let mut cache = state.services.cache.get().await?;

    let bytes = prost::Message::encode_to_vec(rule_result);

    let res = warden_stack::redis::pipe()
        .sadd::<_, _>(cache_key, bytes)
        .ignore()
        .smembers(cache_key)
        .query_async::<Vec<Vec<Vec<u8>>>>(&mut cache)
        .await?;

    let members = res
        .first()
        .ok_or_else(|| anyhow::anyhow!("smembers did not return anything"))?;

    members
        .iter()
        .map(|value| RuleResult::decode(value.as_ref()).map_err(anyhow::Error::new))
        .collect()
}

fn aggregate_rules(
    rule_results: &[RuleResult],
    routing: &RoutingConfiguration,
    rule_result: &RuleResult,
) -> Result<(Vec<TypologyResult>, usize)> {
    let mut typology_result: Vec<TypologyResult> = vec![];
    let mut all_rules_set = HashSet::new();

    routing.messages.iter().for_each(|message| {
        message.typologies.iter().for_each(|typology| {
            let mut set = HashSet::new();

            for rule in typology.rules.iter() {
                set.insert((&rule.id, rule.version()));
                all_rules_set.insert((&rule.id, rule.version()));
            }

            if !set.contains(&(&rule_result.id, rule_result.version.as_str())) {
                return;
            }

            let rule_results: Vec<_> = rule_results
                .iter()
                .filter_map(|value| {
                    if set.contains(&(&value.id, &value.version)) {
                        Some(value.to_owned())
                    } else {
                        None
                    }
                })
                .collect();

            if !rule_results.is_empty() {
                typology_result.push(TypologyResult {
                    id: typology.id.to_owned(),
                    version: typology.version.to_owned(),
                    rule_results,
                    ..Default::default()
                });
            }
        });
    });

    Ok((typology_result, all_rules_set.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warden_core::{
        configuration::routing::{Message, RoutingConfiguration, Rule, Typology},
        message::RuleResult,
    };

    fn create_rule(id: &str, version: &str) -> Rule {
        Rule {
            id: id.to_string(),
            version: Some(version.to_string()),
        }
    }

    fn create_rule_result(id: &str, version: &str) -> RuleResult {
        RuleResult {
            id: id.to_string(),
            version: version.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn returns_empty_when_no_matching_typology() {
        let routing = RoutingConfiguration {
            messages: vec![Message {
                typologies: vec![Typology {
                    id: "T1".to_string(),
                    version: "v1".to_string(),
                    rules: vec![create_rule("R1", "v1")],
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let rule_results = vec![create_rule_result("R2", "v1")];
        let input_rule = create_rule_result("R2", "v1");

        let (result, count) = aggregate_rules(&rule_results, &routing, &input_rule).unwrap();
        assert!(result.is_empty());
        assert_eq!(count, 1); // one rule in routing
    }

    #[test]
    fn returns_typology_with_matching_rule() {
        let routing = RoutingConfiguration {
            messages: vec![Message {
                typologies: vec![Typology {
                    id: "T1".to_string(),
                    version: "v1".to_string(),
                    rules: vec![create_rule("R1", "v1"), create_rule("R2", "v1")],
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let rule_results = vec![
            create_rule_result("R1", "v1"),
            create_rule_result("R2", "v1"),
        ];

        let input_rule = create_rule_result("R1", "v1");

        let (result, count) = aggregate_rules(&rule_results, &routing, &input_rule).unwrap();

        assert_eq!(count, 2); // R1, R2
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "T1");
        assert_eq!(result[0].rule_results.len(), 2);
    }

    #[test]
    fn ignores_unrelated_rules_in_rule_results() {
        let routing = RoutingConfiguration {
            messages: vec![Message {
                typologies: vec![Typology {
                    id: "T1".to_string(),
                    version: "v1".to_string(),
                    rules: vec![create_rule("R1", "v1")],
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let rule_results = vec![
            create_rule_result("R1", "v1"),
            create_rule_result("R99", "v1"), // unrelated
        ];

        let input_rule = create_rule_result("R1", "v1");

        let (result, count) = aggregate_rules(&rule_results, &routing, &input_rule).unwrap();

        assert_eq!(count, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rule_results.len(), 1);
        assert_eq!(result[0].rule_results[0].id, "R1");
    }

    #[test]
    fn handles_multiple_messages_and_typologies() {
        let routing = RoutingConfiguration {
            messages: vec![
                Message {
                    typologies: vec![
                        Typology {
                            id: "T1".to_string(),
                            version: "v1".to_string(),
                            rules: vec![create_rule("R1", "v1")],
                        },
                        Typology {
                            id: "T2".to_string(),
                            version: "v1".to_string(),
                            rules: vec![create_rule("R2", "v1")],
                        },
                    ],
                    ..Default::default()
                },
                Message {
                    typologies: vec![Typology {
                        id: "T3".to_string(),
                        version: "v1".to_string(),
                        rules: vec![create_rule("R1", "v1"), create_rule("R2", "v1")],
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let rule_results = vec![
            create_rule_result("R1", "v1"),
            create_rule_result("R2", "v1"),
        ];
        let input_rule = create_rule_result("R1", "v1");

        let (result, count) = aggregate_rules(&rule_results, &routing, &input_rule).unwrap();

        assert_eq!(count, 2); // R1, R2 appear in multiple typologies, but unique rules are 2
        assert_eq!(result.len(), 2); // T1 (R1) and T3 (R1 & R2)
        assert_eq!(result[0].id, "T1");
        assert_eq!(result[1].id, "T3");
    }
}
