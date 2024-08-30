// Google protobuf Timestamp and Duration types reimplemented because their
// comments are turned into invalid documentation texts and doctest chokes on them. See https://github.com/danburkert/prost/issues/374
// Prost does not seem to have a way yet to remove documentations defined in
// protobuf files. These structs are defined in gogoproto v1.3.1 at https://github.com/gogo/protobuf/tree/v1.3.1/protobuf/google/protobuf

#[cfg(not(feature = "grpc"))]
use core::fmt;
#[cfg(feature = "grpc")]
use std::fmt;

/// A Timestamp represents a point in time independent of any time zone or local
/// calendar, encoded as a count of seconds and fractions of seconds at
/// nanosecond resolution. The count is relative to an epoch at UTC midnight on
/// January 1, 1970, in the proleptic Gregorian calendar which extends the
/// Gregorian calendar backwards to year one.
///
/// All minutes are 60 seconds long. Leap seconds are "smeared" so that no leap
/// second table is needed for interpretation, using a [24-hour linear
/// smear](https://developers.google.com/time/smear).
///
/// The range is from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z. By
/// restricting to that range, we ensure that we can convert to and from [RFC
/// 3339](https://www.ietf.org/rfc/rfc3339.txt) date strings.
#[derive(Clone, PartialEq, ::prost::Message)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        from = "crate::serializers::timestamp::Rfc3339",
        into = "crate::serializers::timestamp::Rfc3339"
    )
)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

/// A Duration represents a signed, fixed-length span of time represented
/// as a count of seconds and fractions of seconds at nanosecond
/// resolution. It is independent of any calendar and concepts like "day"
/// or "month". It is related to Timestamp in that the difference between
/// two Timestamp values is a Duration and it can be added or subtracted
/// from a Timestamp. Range is approximately +-10,000 years.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Duration {
    /// Signed seconds of the span of time. Must be from -315,576,000,000
    /// to +315,576,000,000 inclusive. Note: these bounds are computed from:
    /// 60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Signed fractions of a second at nanosecond resolution of the span
    /// of time. Durations less than one second are represented with a 0
    /// `seconds` field and a positive or negative `nanos` field. For durations
    /// of one second or more, a non-zero value for the `nanos` field must be
    /// of the same sign as the `seconds` field. Must be from -999,999,999
    /// to +999,999,999 inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}
#[cfg(feature = "serde")]
impl serde::Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let total_nanos = self.seconds * 1_000_000_000 + self.nanos as i64;
        serializer.serialize_i64(total_nanos)
    }
}
#[cfg(feature = "serde")]
struct DurationVisitor;
#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a nanosecond representation of a duration")
    }

    fn visit_i128<E>(self, value: i128) -> Result<Duration, E>
    where
        E: serde::de::Error,
    {
        let seconds = (value / 1_000_000_000) as i64;
        let nanos = (value % 1_000_000_000) as i32;
        Ok(Duration { seconds, nanos })
    }

    fn visit_str<E>(self, value: &str) -> Result<Duration, E>
    where
        E: serde::de::Error,
    {
        let value = value.parse::<i128>().map_err(E::custom)?;
        self.visit_i128(value)
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DurationVisitor)
    }
}
