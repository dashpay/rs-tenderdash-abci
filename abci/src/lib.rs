mod application;
#[cfg(feature = "server")]
mod server;

use std::io;

pub use application::{Application, RequestDispatcher};
use prost::{DecodeError, EncodeError};
pub use server::{start_server, BindAddress, Server};
pub use tenderdash_proto as proto;

/// Errors that may happen during protobuf communication
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection error")]
    Connection(#[from] io::Error),
    #[error("cannot decode protobuf message")]
    Decode(#[from] DecodeError),
    #[error("cannot encode protobuf message")]
    Encode(#[from] EncodeError),
}
