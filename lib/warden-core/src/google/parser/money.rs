use crate::google::r#type::Money;

/// If money cannot be created
#[derive(Debug, PartialEq)]
pub enum MoneyError {
    /// Invalid currency code
    InvalidCurrencyCode,
}

impl<T> TryFrom<(f64, T)> for Money
where
    T: AsRef<str>,
{
    type Error = MoneyError;

    fn try_from((value, ccy): (f64, T)) -> Result<Self, Self::Error> {
        let currency_code = ccy.as_ref();

        let is_valid =
            currency_code.len() == 3 && currency_code.chars().all(|c| c.is_ascii_uppercase());

        if !is_valid {
            return Err(MoneyError::InvalidCurrencyCode);
        }

        let units = value.trunc() as i64;
        let nanos_raw = ((value - units as f64) * 1_000_000_000.0).round() as i32;

        let (units, nanos) = if units > 0 && nanos_raw < 0 {
            (units - 1, nanos_raw + 1_000_000_000)
        } else if units < 0 && nanos_raw > 0 {
            (units + 1, nanos_raw - 1_000_000_000)
        } else {
            (units, nanos_raw)
        };

        Ok(Money {
            currency_code: currency_code.to_string(),
            units,
            nanos,
        })
    }
}

impl From<Money> for f64 {
    fn from(m: Money) -> Self {
        m.units as f64 + (m.nanos as f64) / 1_000_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_valid_money_conversion() {
        let money = Money::try_from((1.75, "USD")).unwrap();
        assert_eq!(
            money,
            Money {
                currency_code: "USD".to_string(),
                units: 1,
                nanos: 750_000_000
            }
        );

        let money = Money::try_from((-1.75, "EUR")).unwrap();
        assert_eq!(
            money,
            Money {
                currency_code: "EUR".to_string(),
                units: -1,
                nanos: -750_000_000
            }
        );

        let money = Money::try_from((0.333_333_333, "JPY")).unwrap();
        assert_eq!(
            money,
            Money {
                currency_code: "JPY".to_string(),
                units: 0,
                nanos: 333_333_333
            }
        );
    }

    #[test]
    fn test_sign_correction() {
        let m = Money::try_from((1.000_000_001, "USD")).unwrap();
        assert_eq!(m.units, 1);
        assert_eq!(m.nanos, 1);

        let m = Money::try_from((-1.000_000_001, "USD")).unwrap();
        assert_eq!(m.units, -1);
        assert_eq!(m.nanos, -1);

        let m = Money::try_from((0.000_000_001, "USD")).unwrap();
        assert_eq!(m.units, 0);
        assert_eq!(m.nanos, 1);

        let m = Money::try_from((-0.000_000_001, "USD")).unwrap();
        assert_eq!(m.units, 0);
        assert_eq!(m.nanos, -1);
    }

    #[test]
    fn test_invalid_currency_code() {
        let cases = ["usd", "US", "USDD", "12A", "€€€", ""];

        for &code in &cases {
            let result = Money::try_from((1.0, code));
            assert_eq!(result, Err(MoneyError::InvalidCurrencyCode));
        }
    }

    #[test]
    fn test_money_to_f64_positive() {
        let money = Money {
            currency_code: "USD".to_string(),
            units: 1,
            nanos: 750_000_000,
        };
        let value: f64 = money.into();
        assert_eq!(value, 1.75);
    }

    #[test]
    fn test_money_to_f64_negative() {
        let money = Money {
            currency_code: "EUR".to_string(),
            units: -1,
            nanos: -250_000_000,
        };
        let value: f64 = money.into();
        assert_eq!(value, -1.25);
    }

    #[test]
    fn test_money_to_f64_zero() {
        let money = Money {
            currency_code: "JPY".to_string(),
            units: 0,
            nanos: 0,
        };
        let value: f64 = money.into();
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_money_to_f64_fractional_positive() {
        let money = Money {
            currency_code: "USD".to_string(),
            units: 0,
            nanos: 1,
        };
        let value: f64 = money.into();
        assert_eq!(value, 1.0e-9);
    }

    #[test]
    fn test_money_to_f64_fractional_negative() {
        let money = Money {
            currency_code: "USD".to_string(),
            units: 0,
            nanos: -1,
        };
        let value: f64 = money.into();
        assert_eq!(value, -1.0e-9);
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = 1_234.567_890_123;
        let money = Money::try_from((original, "USD")).unwrap();
        let back: f64 = money.into();
        assert!(
            (original - back).abs() < 1e-9,
            "got {}, expected {}",
            back,
            original
        );
    }
}
