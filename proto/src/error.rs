//! This module defines the various errors that be raised during Protobuf
//! conversions.
extern crate std;
use core::{convert::TryFrom, fmt::Display, num::TryFromIntError};

use prost::{DecodeError, EncodeError};

use crate::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// error converting message type into domain type
    #[error("error converting message type into domain type: {0}")]
    TryFromProtobuf(String),

    /// error encoding message into buffer
    #[error("error encoding message into buffer: {0:?}")]
    EncodeMessage(EncodeError),
    /// error decoding buffer into message
    #[error("error decoding buffer into message: {0:?}")]
    DecodeMessage(DecodeError),
    /// error decoding buffer into message
    #[error("error building canonical message: {0}")]
    CreateCanonical(String),
    /// error parsing encoded length
    #[error("error parsing encoded length: {0:?}")]
    ParseLength(TryFromIntError),
}

impl Error {
    pub fn try_from<Raw, T, E>(e: E) -> Error
    where
        E: Display,
        T: TryFrom<Raw, Error = E>,
    {
        Error::TryFromProtobuf(format!("{e}"))
    }
}
