mod application;
#[cfg(feature = "server")]
mod server;

use std::io;

pub use application::{Application, RequestDispatcher};
use prost::{DecodeError, EncodeError};
pub use server::{start_tcp, start_unix};
pub use tenderdash_proto::abci as abci_proto;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection error")]
    Connection(#[from] io::Error),
    #[error("cannot decode protobuf message")]
    Decode(#[from] DecodeError),
    #[error("cannot encode protobuf message")]
    Encode(#[from] EncodeError),
}
