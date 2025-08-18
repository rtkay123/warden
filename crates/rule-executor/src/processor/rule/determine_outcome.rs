use tracing::trace;
use warden_core::{configuration::rule::Band, message::RuleResult};

pub(super) fn determine_outcome(value: i64, bands: &[Band], rule_result: &mut RuleResult) {
    trace!("calculating outcome");
    for band in bands {
        let value_f64 = value as f64;

        if band.lower_limit.is_none_or(|lower| value_f64 >= lower)
            && band.upper_limit.is_none_or(|upper| value_f64 < upper)
        {
            rule_result.sub_rule_ref = band.sub_rule_ref.to_owned();
            rule_result.reason = band.reason.to_owned();
            break;
        }
    }
}
