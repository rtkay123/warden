use anyhow::Result;
use tracing::warn;
use warden_core::{configuration::typology::TypologyConfiguration, message::TypologyResult};

pub(super) fn evaluate_expression(
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

#[cfg(test)]
mod tests {
    use warden_core::{
        configuration::typology::{Expression, Operator, Term, TypologyRule, TypologyRuleWeight},
        message::RuleResult,
    };

    use super::*;

    fn make_rule_result(id: &str, version: &str, sub_ref: &str) -> RuleResult {
        RuleResult {
            id: id.to_string(),
            version: version.to_string(),
            sub_rule_ref: sub_ref.to_string(),
            ..Default::default()
        }
    }

    fn make_rule_value(id: &str, version: &str, ref_name: &str, weight: f64) -> TypologyRule {
        TypologyRule {
            id: id.to_string(),
            version: version.to_string(),
            wghts: vec![TypologyRuleWeight {
                r#ref: ref_name.to_string(),
                wght: weight,
            }],
        }
    }

    fn make_expression(terms: Vec<(&str, &str)>, op: Operator) -> Expression {
        Expression {
            terms: terms
                .into_iter()
                .map(|(id, version)| Term {
                    id: id.to_string(),
                    version: version.to_string(),
                })
                .collect(),
            operator: op.into(),
        }
    }

    #[test]
    fn test_add_operator_multiple_terms() {
        let mut typology_result = TypologyResult {
            rule_results: vec![
                make_rule_result("R1", "v1", "sub1"),
                make_rule_result("R2", "v1", "sub2"),
            ],
            ..Default::default()
        };

        let config = TypologyConfiguration {
            expression: Some(make_expression(
                vec![("R1", "v1"), ("R2", "v1")],
                Operator::Add,
            )),
            rules: vec![
                make_rule_value("R1", "v1", "sub1", 10.0),
                make_rule_value("R2", "v1", "sub2", 5.0),
            ],
            ..Default::default()
        };

        let result = evaluate_expression(&mut typology_result, &config).unwrap();
        assert_eq!(result, 15.0);
    }

    #[test]
    fn test_missing_rule_result_returns_zero() {
        let mut typology_result = TypologyResult {
            rule_results: vec![make_rule_result("R1", "v1", "sub1")],
            ..Default::default()
        };

        let config = TypologyConfiguration {
            expression: Some(make_expression(
                vec![("R1", "v1"), ("R2", "v1")],
                Operator::Add,
            )),
            rules: vec![make_rule_value("R1", "v1", "sub1", 10.0)],
            ..Default::default()
        };

        let result = evaluate_expression(&mut typology_result, &config).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_missing_weight_defaults_to_zero() {
        let mut typology_result = TypologyResult {
            rule_results: vec![make_rule_result("R1", "v1", "subX")], // sub_ref doesn't match
            ..Default::default()
        };

        let config = TypologyConfiguration {
            expression: Some(make_expression(vec![("R1", "v1")], Operator::Add)),
            rules: vec![make_rule_value("R1", "v1", "sub1", 10.0)], // different ref
            ..Default::default()
        };

        let result = evaluate_expression(&mut typology_result, &config).unwrap();
        assert_eq!(result, 0.0);
    }
}
