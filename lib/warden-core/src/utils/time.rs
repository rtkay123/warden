use crate::google::protobuf::Timestamp;
use time::OffsetDateTime;
impl From<OffsetDateTime> for Timestamp {
    fn from(dt: OffsetDateTime) -> Self {
        Timestamp {
            seconds: dt.unix_timestamp(),
            nanos: dt.nanosecond() as i32,
        }
    }
}

impl TryFrom<Timestamp> for OffsetDateTime {
    type Error = time::Error;

    fn try_from(timestamp: Timestamp) -> Result<Self, Self::Error> {
        let seconds = timestamp.seconds;
        let nanos = timestamp.nanos as i64;
        let nanoseconds = nanos % 1_000_000_000;
        let d = OffsetDateTime::from_unix_timestamp(seconds)?
            + time::Duration::nanoseconds(nanoseconds);
        Ok(d)
    }
}

#[cfg(test)]
mod tests {
    use crate::google::protobuf::Timestamp;
    use std::convert::TryFrom;
    use time::{Duration, OffsetDateTime};

    #[test]
    fn test_offset_datetime_to_timestamp() {
        // Given a specific OffsetDateTime
        let dt = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap()
            + Duration::nanoseconds(123_456_789);

        // Convert to Timestamp
        let timestamp: Timestamp = dt.into();

        // Assert the seconds and nanos
        assert_eq!(timestamp.seconds, 1_600_000_000);
        assert_eq!(timestamp.nanos, 123_456_789);
    }

    #[test]
    fn test_timestamp_to_offset_datetime() {
        // Given a Timestamp
        let timestamp = Timestamp {
            seconds: 1_600_000_000,
            nanos: 123_456_789,
        };

        // Convert to OffsetDateTime
        let dt = OffsetDateTime::try_from(timestamp).unwrap();

        // Assert the conversion is accurate
        assert_eq!(dt.unix_timestamp(), 1_600_000_000);
        assert_eq!(dt.nanosecond(), 123_456_789);
    }

    #[test]
    fn test_negative_timestamp_to_offset_datetime() {
        // Given a negative Timestamp (before Unix epoch)
        let timestamp = Timestamp {
            seconds: -1_600_000_000,
            nanos: 987_654_321,
        };

        // Convert to OffsetDateTime
        let dt = OffsetDateTime::try_from(timestamp).unwrap();

        // Assert the conversion is accurate for negative timestamps
        assert_eq!(dt.unix_timestamp(), -1_600_000_000);
        assert_eq!(dt.nanosecond(), 987_654_321);
    }

    #[test]
    fn test_offset_datetime_to_timestamp_and_back() {
        // Given a specific OffsetDateTime
        let dt = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap()
            + Duration::nanoseconds(987_654_321);

        // Convert to Timestamp
        let timestamp: Timestamp = dt.into();

        // Convert it back to OffsetDateTime
        let dt_back = OffsetDateTime::try_from(timestamp).unwrap();

        // Assert the two dates are the same
        assert_eq!(dt.unix_timestamp(), dt_back.unix_timestamp());
        assert_eq!(dt.nanosecond(), dt_back.nanosecond());
    }
}
