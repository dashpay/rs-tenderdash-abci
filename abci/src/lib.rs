//! Tenderdash ABCI Application library.
//!
//! ABCI Application is responsible for storage and implementation of
//! application state.
//!
//! Use [[start_server]] to create a new server that will accept connections
//! from Tenderdash.
//!
//! Run [Server::handle_connection()] in a loop to handle incoming server
//! connections.
//!
//! Implement the [application::Application] trait with custom logic for
//! blockchain processing. Expect messages defined in [proto::abci] crate.

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
