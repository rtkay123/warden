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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_band(lower: Option<f64>, upper: Option<f64>, sub_ref: &str, reason: &str) -> Band {
        Band {
            lower_limit: lower,
            upper_limit: upper,
            sub_rule_ref: sub_ref.to_string(),
            reason: reason.to_string(),
        }
    }

    #[test]
    fn matches_band_within_limits() {
        let bands = vec![
            make_band(Some(0.0), Some(10.0), "A", "Between 0 and 10"),
            make_band(Some(10.0), Some(20.0), "B", "Between 10 and 20"),
        ];
        let mut rule_result = RuleResult::default();

        determine_outcome(5, &bands, &mut rule_result);

        assert_eq!(rule_result.sub_rule_ref, "A");
        assert_eq!(rule_result.reason, "Between 0 and 10");
    }

    #[test]
    fn matches_band_lower_inclusive_upper_exclusive() {
        let bands = vec![
            make_band(Some(0.0), Some(10.0), "A", "Between 0 and 10"),
            make_band(Some(10.0), Some(20.0), "B", "Between 10 and 20"),
        ];
        let mut rule_result = RuleResult::default();

        determine_outcome(10, &bands, &mut rule_result);

        assert_eq!(rule_result.sub_rule_ref, "B");
        assert_eq!(rule_result.reason, "Between 10 and 20");
    }

    #[test]
    fn no_match_when_above_all_bands() {
        let bands = vec![
            make_band(Some(0.0), Some(10.0), "A", "Between 0 and 10"),
            make_band(Some(10.0), Some(20.0), "B", "Between 10 and 20"),
        ];
        let mut rule_result = RuleResult::default();

        determine_outcome(30, &bands, &mut rule_result);

        assert_eq!(rule_result, RuleResult::default());
    }

    #[test]
    fn match_when_no_upper_limit() {
        let bands = vec![make_band(Some(0.0), None, "A", "Above 0")];
        let mut rule_result = RuleResult::default();

        determine_outcome(100, &bands, &mut rule_result);

        assert_eq!(rule_result.sub_rule_ref, "A");
        assert_eq!(rule_result.reason, "Above 0");
    }

    #[test]
    fn match_when_no_lower_limit() {
        let bands = vec![make_band(None, Some(50.0), "A", "Below 50")];
        let mut rule_result = RuleResult::default();

        determine_outcome(-10, &bands, &mut rule_result);

        assert_eq!(rule_result.sub_rule_ref, "A");
        assert_eq!(rule_result.reason, "Below 50");
    }

    #[test]
    fn match_when_no_limits() {
        let bands = vec![make_band(None, None, "A", "Any value")];
        let mut rule_result = RuleResult::default();

        determine_outcome(9999, &bands, &mut rule_result);

        assert_eq!(rule_result.sub_rule_ref, "A");
        assert_eq!(rule_result.reason, "Any value");
    }

    #[test]
    fn stops_after_first_match() {
        let bands = vec![
            make_band(None, None, "A", "Any value"),
            make_band(None, None, "B", "Second band"),
        ];
        let mut rule_result = RuleResult::default();

        determine_outcome(5, &bands, &mut rule_result);

        assert_eq!(rule_result.sub_rule_ref, "A");
        assert_eq!(rule_result.reason, "Any value");
    }
}
