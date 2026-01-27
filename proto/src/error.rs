//! This module defines the various errors that be raised during Protobuf
//! conversions.
#[cfg(not(feature = "std"))]
use core::{convert::TryFrom, fmt::Display, num::TryFromIntError};
#[cfg(feature = "std")]
use std::{fmt::Display, num::TryFromIntError};

use prost::{DecodeError, EncodeError};
use thiserror::Error as ThisError;

use crate::prelude::*;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("error converting time: {reason}")]
    TimeConversion { reason: String },

    #[error("error converting message type into domain type: {reason}")]
    TryFromProtobuf { reason: String },

    // Keep messages consistent with previous flex_error display strings
    #[error("error encoding message into buffer")]
    EncodeMessage(EncodeError),

    #[error("error decoding buffer into message")]
    DecodeMessage(DecodeError),

    #[error("error parsing encoded length")]
    ParseLength(TryFromIntError),
}

impl Error {
    // Backwards-compatible constructors mirroring flex_error-generated fns
    pub fn time_conversion(reason: String) -> Self {
        Self::TimeConversion { reason }
    }

    pub fn try_from_protobuf(reason: String) -> Self {
        Self::TryFromProtobuf { reason }
    }

    pub fn encode_message(err: EncodeError) -> Self {
        Self::EncodeMessage(err)
    }

    pub fn decode_message(err: DecodeError) -> Self {
        Self::DecodeMessage(err)
    }

    pub fn parse_length(err: TryFromIntError) -> Self {
        Self::ParseLength(err)
    }

    pub fn try_from<Raw, T, E>(e: E) -> Error
    where
        E: Display,
        T: TryFrom<Raw, Error = E>,
    {
        Error::try_from_protobuf(format!("{e}"))
    }
}
