//! This module defines the various errors that be raised during Protobuf
//! conversions.
#[cfg(not(feature = "grpc"))]
use core::{convert::TryFrom, fmt::Display, num::TryFromIntError};
#[cfg(feature = "grpc")]
use std::{fmt::Display, num::TryFromIntError};

use flex_error::{define_error, DisplayOnly};
use prost::{DecodeError, EncodeError};

use crate::prelude::*;

define_error! {
    Error {
        TimeConversion
            { reason: String }
            | e | {
            format!("error converting time: {}", e.reason)
        },
        TryFromProtobuf
            { reason: String }
            | e | {
                format!("error converting message type into domain type: {}",
                    e.reason)
            },

        EncodeMessage
            [ DisplayOnly<EncodeError> ]
            | _ | { "error encoding message into buffer" },

        DecodeMessage
            [ DisplayOnly<DecodeError> ]
            | _ | { "error decoding buffer into message" },

        ParseLength
            [ DisplayOnly<TryFromIntError> ]
            | _ | { "error parsing encoded length" },
    }
}

impl Error {
    pub fn try_from<Raw, T, E>(e: E) -> Error
    where
        E: Display,
        T: TryFrom<Raw, Error = E>,
    {
        Error::try_from_protobuf(format!("{e}"))
    }
}
