//! Time conversion traits and functions

use crate::google::protobuf::Timestamp;
pub trait ToMillis {
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
    fn to_millis(&self) -> u64;

    #[deprecated = "use `to_millis` instead"]
    fn to_milis(&self) -> u64 {
        self.to_millis()
    }
}

impl ToMillis for Timestamp {
    /// Convert protobuf timestamp into miliseconds since epoch
    fn to_millis(&self) -> u64 {
        chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32)
            .unwrap()
            .to_utc()
            .timestamp_millis()
            .try_into()
            .expect("timestamp value out of u64 range")
    }
}

pub trait FromMillis {
    /// Create protobuf timestamp from miliseconds since epoch
    ///
    /// Note there is a resolution difference, as timestamp uses nanoseconds
    ///
    /// # Arguments
    ///
    /// * millis - time since epoch, in miliseconds; must fit `i64` type
    fn from_millis(millis: u64) -> Self;

    #[deprecated = "use `from_millis` instead"]
    fn from_milis(millis: u64) -> Self
    where
        Self: Sized,
    {
        Self::from_millis(millis)
    }
}

impl FromMillis for Timestamp {
    /// Create protobuf timestamp from miliseconds since epoch
    ///
    /// Note there is a resolution difference, as timestamp uses nanoseconds
    ///
    /// # Panics
    ///  
    /// Panics when `millis` don't fit `i64` type
    fn from_millis(millis: u64) -> Self {
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
