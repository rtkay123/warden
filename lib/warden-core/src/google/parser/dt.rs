use crate::google::protobuf::Timestamp;

#[cfg(feature = "message")]
mod date {
    use super::*;
    use crate::google::r#type::Date;

    impl From<time::OffsetDateTime> for Date {
        fn from(dt: time::OffsetDateTime) -> Self {
            Self {
                year: dt.year(),
                month: dt.month() as i32,
                day: dt.day() as i32,
            }
        }
    }

    impl From<time::Date> for Date {
        fn from(value: time::Date) -> Self {
            Self {
                year: value.year(),
                month: value.month() as i32,
                day: value.day() as i32,
            }
        }
    }

    impl TryFrom<Date> for time::Date {
        type Error = time::Error;

        fn try_from(value: Date) -> Result<Self, Self::Error> {
            Ok(Self::from_calendar_date(
                value.year,
                time::Month::try_from(value.month as u8)?,
                value.day as u8,
            )?)
        }
    }

    impl std::str::FromStr for Date {
        type Err = time::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let date =
                time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map(Date::from);

            match date {
                Ok(dt) => Ok(dt),
                Err(_e) => {
                    let my_format = time::macros::format_description!("[year]-[month]-[day]");
                    let date = time::Date::parse(s, &my_format)?;
                    Ok(Date::from(date))
                }
            }
        }
    }

    impl TryFrom<String> for Date {
        type Error = time::Error;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            <Date as std::str::FromStr>::from_str(&value)
        }
    }

    impl TryFrom<DateItem> for Date {
        type Error = time::Error;

        fn try_from(value: DateItem) -> Result<Self, Self::Error> {
            match value {
                DateItem::String(ref string) => <Date as std::str::FromStr>::from_str(string),
                #[cfg(feature = "message")]
                DateItem::Date { year, month, day } => Ok(Date { year, month, day }),
                DateItem::Timestamp { seconds, nanos } => {
                    let odt = time::OffsetDateTime::try_from(crate::google::protobuf::Timestamp {
                        seconds,
                        nanos,
                    })?;
                    Ok(Self {
                        year: odt.year(),
                        month: odt.month() as i32,
                        day: odt.day() as i32,
                    })
                }
            }
        }
    }

    impl From<Date> for String {
        fn from(value: Date) -> Self {
            let prepend = |value: i32| -> String {
                match value.lt(&10) {
                    true => format!("0{}", value),
                    false => value.to_string(),
                }
            };
            format!(
                "{}-{}-{}",
                value.year,
                prepend(value.month),
                prepend(value.day),
            )
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
/// Date utility
#[derive(Clone, Debug)]
pub enum DateItem {
    /// string
    String(String),
    /// ts
    Timestamp { seconds: i64, nanos: i32 },
    /// date
    #[cfg(feature = "message")]
    Date { year: i32, month: i32, day: i32 },
}

impl TryFrom<DateItem> for Timestamp {
    type Error = time::Error;

    fn try_from(value: DateItem) -> Result<Self, Self::Error> {
        match value {
            DateItem::String(ref string) => <Timestamp as std::str::FromStr>::from_str(string),
            #[cfg(feature = "message")]
            DateItem::Date { year, month, day } => {
                let date = time::Date::try_from(crate::google::r#type::Date { year, month, day })?;
                let time = time::Time::MIDNIGHT;
                let offset = time::UtcOffset::UTC;
                Ok(date.with_time(time).assume_offset(offset).into())
            }
            DateItem::Timestamp { seconds, nanos } => Ok(Self { seconds, nanos }),
        }
    }
}

impl From<time::OffsetDateTime> for Timestamp {
    fn from(dt: time::OffsetDateTime) -> Self {
        Timestamp {
            seconds: dt.unix_timestamp(),
            nanos: dt.nanosecond() as i32,
        }
    }
}

impl From<Timestamp> for String {
    fn from(value: Timestamp) -> Self {
        let odt = time::OffsetDateTime::try_from(value).expect("invalid date");
        odt.format(&time::format_description::well_known::Rfc3339)
            .expect("format is not rfc3339")
    }
}

impl std::str::FromStr for Timestamp {
    type Err = time::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let timestamp =
            time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)?;

        Ok(Timestamp::from(timestamp))
    }
}

impl TryFrom<String> for Timestamp {
    type Error = time::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Timestamp as std::str::FromStr>::from_str(&value)
    }
}

impl TryFrom<Timestamp> for time::OffsetDateTime {
    type Error = time::Error;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        let dt = time::OffsetDateTime::from_unix_timestamp(value.seconds)?;

        Ok(dt.replace_nanosecond(value.nanos as u32)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Duration, OffsetDateTime};

    #[test]
    fn test_offsetdatetime_to_timestamp() {
        let now = OffsetDateTime::now_utc();
        let timestamp: Timestamp = now.into();

        assert_eq!(timestamp.seconds, now.unix_timestamp());
        assert_eq!(timestamp.nanos, now.nanosecond() as i32);
    }

    #[test]
    fn test_timestamp_to_offsetdatetime() {
        let now = OffsetDateTime::now_utc();
        let timestamp: Timestamp = now.into();
        let dt: OffsetDateTime = timestamp.try_into().unwrap();

        assert_eq!(dt, now);
    }

    #[test]
    fn test_timestamp_to_offsetdatetime_with_nanos() {
        let now = OffsetDateTime::now_utc();
        let nanos = 123456789;
        let dt = now + Duration::nanoseconds(nanos);
        let timestamp: Timestamp = dt.into();
        let dt_from_timestamp: OffsetDateTime = timestamp.try_into().unwrap();

        assert_eq!(dt_from_timestamp, dt);
    }

    #[test]
    fn test_timestamp_to_offsetdatetime_with_negative_nanos() {
        let now = OffsetDateTime::now_utc();
        let nanos = -123456789;
        let dt = now + Duration::nanoseconds(nanos);
        let timestamp: Timestamp = dt.into();
        let dt_from_timestamp: OffsetDateTime = timestamp.try_into().unwrap();

        assert_eq!(dt_from_timestamp, dt);
    }

    #[test]
    fn test_timestamp_to_offsetdatetime_invalid_seconds() {
        let timestamp = Timestamp {
            seconds: i64::MIN,
            nanos: 0,
        };
        let result: Result<OffsetDateTime, time::Error> = timestamp.try_into();
        assert!(result.is_err());
    }
}
