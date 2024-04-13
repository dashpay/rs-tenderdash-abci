//! Serialize/deserialize Timestamp type from and into string:
#[cfg(not(feature = "grpc"))]
use core::fmt::{self, Debug};
#[cfg(feature = "grpc")]
use std::fmt::{self, Debug};

use serde::{de::Error as _, ser::Error, Deserialize, Deserializer, Serialize, Serializer};
use time::{
    format_description::well_known::Rfc3339 as Rfc3339Format, macros::offset, OffsetDateTime,
};

use crate::{google::protobuf::Timestamp, prelude::*};

/// Helper struct to serialize and deserialize Timestamp into an
/// RFC3339-compatible string This is required because the serde `with`
/// attribute is only available to fields of a struct but not the whole struct.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rfc3339(#[serde(with = "crate::serializers::timestamp")] Timestamp);

impl From<Timestamp> for Rfc3339 {
    fn from(value: Timestamp) -> Self {
        Rfc3339(value)
    }
}
impl From<Rfc3339> for Timestamp {
    fn from(value: Rfc3339) -> Self {
        value.0
    }
}

pub trait ToMilis {
    /// Convert protobuf timestamp into miliseconds since epoch

    /// Note there is a resolution difference, as timestamp uses nanoseconds
    ///
    /// # Arguments
    ///
    /// * millis - time since epoch, in miliseconds
    ///
    /// # Panics
    ///  
    /// Panics when timestamp doesn't fit `u64` type
    fn to_milis(&self) -> u64;
}

impl ToMilis for Timestamp {
    /// Convert protobuf timestamp into miliseconds since epoch
    fn to_milis(&self) -> u64 {
        chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32)
            .unwrap()
            .to_utc()
            .timestamp_millis()
            .try_into()
            .expect("timestamp value out of u64 range")
    }
}

pub trait FromMilis {
    /// Create protobuf timestamp from miliseconds since epoch
    ///
    /// Note there is a resolution difference, as timestamp uses nanoseconds
    ///
    /// # Arguments
    ///
    /// * millis - time since epoch, in miliseconds; must fit `i64` type
    fn from_milis(millis: u64) -> Self;
}

impl FromMilis for Timestamp {
    /// Create protobuf timestamp from miliseconds since epoch
    ///
    /// Note there is a resolution difference, as timestamp uses nanoseconds
    ///
    /// # Panics
    ///  
    /// Panics when `millis` don't fit `i64` type
    fn from_milis(millis: u64) -> Self {
        let dt = chrono::DateTime::from_timestamp_millis(
            millis
                .try_into()
                .expect("milliseconds timestamp out of i64 range"),
        )
        .expect("cannot parse timestamp")
        .to_utc();

        Self {
            nanos: dt.timestamp_subsec_nanos() as i32,
            seconds: dt.timestamp(),
        }
    }
}

/// Deserialize string into Timestamp
pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value_string = String::deserialize(deserializer)?;
    let t = OffsetDateTime::parse(&value_string, &Rfc3339Format).map_err(D::Error::custom)?;
    let t = t.to_offset(offset!(UTC));
    if !matches!(t.year(), 1..=9999) {
        return Err(D::Error::custom("date is out of range"));
    }
    let seconds = t.unix_timestamp();
    // Safe to convert to i32 because .nanosecond()
    // is guaranteed to return a value in 0..1_000_000_000 range.
    let nanos = t.nanosecond() as i32;
    Ok(Timestamp { seconds, nanos })
}

/// Serialize from Timestamp into string
pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.nanos < 0 || value.nanos > 999_999_999 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    let total_nanos = value.seconds as i128 * 1_000_000_000 + value.nanos as i128;
    let datetime = OffsetDateTime::from_unix_timestamp_nanos(total_nanos)
        .map_err(|_| S::Error::custom("invalid time"))?;
    to_rfc3339_nanos(datetime).serialize(serializer)
}

/// Serialization helper for converting an [`OffsetDateTime`] object to a
/// string.
///
/// This reproduces the behavior of Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
pub fn to_rfc3339_nanos(t: OffsetDateTime) -> String {
    // Can't use OffsetDateTime::format because the feature enabling it
    // currently requires std (https://github.com/time-rs/time/issues/400)

    // Preallocate enough string capacity to fit the shortest possible form,
    // yyyy-mm-ddThh:mm:ssZ
    let mut buf = String::with_capacity(20);

    fmt_as_rfc3339_nanos(t, &mut buf).unwrap();

    buf
}

/// Helper for formatting an [`OffsetDateTime`] value.
///
/// This function can be used to efficiently format date-time values
/// in [`Display`] or [`Debug`] implementations.
///
/// The format reproduces Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
///
/// [`Display`]: fmt::Display
/// [`Debug`]: fmt::Debug
pub fn fmt_as_rfc3339_nanos(t: OffsetDateTime, f: &mut impl fmt::Write) -> fmt::Result {
    let t = t.to_offset(offset!(UTC));
    let nanos = t.nanosecond();
    if nanos == 0 {
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",
            year = t.year(),
            month = t.month() as u8,
            day = t.day(),
            hour = t.hour(),
            minute = t.minute(),
            second = t.second(),
        )
    } else {
        let mut secfrac = nanos;
        let mut secfrac_width = 9;
        while secfrac % 10 == 0 {
            secfrac /= 10;
            secfrac_width -= 1;
        }
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{secfrac:0sfw$}Z",
            year = t.year(),
            month = t.month() as u8,
            day = t.day(),
            hour = t.hour(),
            minute = t.minute(),
            second = t.second(),
            secfrac = secfrac,
            sfw = secfrac_width,
        )
    }
}

#[allow(warnings)]
#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::google::protobuf::Timestamp;

    // The Go code with which the following timestamps
    // were tested is as follows:
    //
    // ```go
    // package main
    //
    // import (
    //     "fmt"
    //     "time"
    // )
    //
    // func main() {
    //     timestamps := []string{
    //         "1970-01-01T00:00:00Z",
    //         "0001-01-01T00:00:00Z",
    //         "2020-09-14T16:33:00Z",
    //         "2020-09-14T16:33:00.1Z",
    //         "2020-09-14T16:33:00.211914212Z",
    //         "2020-09-14T16:33:54.21191421Z",
    //         "2021-01-07T20:25:56.045576Z",
    //         "2021-01-07T20:25:57.039219Z",
    //         "2021-01-07T20:26:05.00509Z",
    //         "2021-01-07T20:26:05.005096Z",
    //         "2021-01-07T20:26:05.0005096Z",
    //     }
    //     for _, timestamp := range timestamps {
    //         ts, err := time.Parse(time.RFC3339Nano, timestamp)
    //         if err != nil {
    //             panic(err)
    //         }
    //         tss := ts.Format(time.RFC3339Nano)
    //         if timestamp != tss {
    //             panic(fmt.Sprintf("\nExpected : %s\nActual   : %s", timestamp, tss))
    //         }
    //     }
    //     fmt.Println("All good!")
    // }
    // ```
    #[test]
    fn json_timestamp_precision() {
        let test_timestamps = vec![
            "1970-01-01T00:00:00Z",
            "0001-01-01T00:00:00Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
            "2020-09-14T16:33:54.21191421Z",
            "2021-01-07T20:25:56.045576Z",
            "2021-01-07T20:25:57.039219Z",
            "2021-01-07T20:26:05.00509Z",
            "2021-01-07T20:26:05.005096Z",
            "2021-01-07T20:26:05.0005096Z",
        ];

        for timestamp in test_timestamps {
            let json = format!("\"{}\"", timestamp);
            let rfc = serde_json::from_str::<Rfc3339>(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&rfc).unwrap());
        }
    }

    #[test]
    fn timestamp_from_to() {
        let time_ms = 1687848809533;

        let from = Timestamp::from_milis(time_ms);
        let to = from.to_milis();

        assert_eq!(to, time_ms);
    }

    #[test]
    #[should_panic]
    fn timestamp_millis_out_of_range() {
        let time_ms = u64::MAX - 1;

        let from = Timestamp::from_milis(time_ms);
    }

    #[test]
    #[should_panic]
    fn timestamp_negative() {
        let ts = Timestamp {
            nanos: 1000,
            seconds: -12,
        };

        let to = ts.to_milis();
    }
}
