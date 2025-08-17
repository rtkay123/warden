use anyhow::Result;
use std::collections::HashSet;

use warden_core::{
    configuration::routing::RoutingConfiguration,
    message::{RuleResult, TypologyResult},
};

pub(super) fn aggregate_rules(
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
