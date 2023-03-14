mod application;
#[cfg(feature = "client")]
mod client;
pub mod error;
#[cfg(feature = "server")]
pub mod server;

// Common exports
// Example applications
#[cfg(feature = "echo-app")]
pub use application::echo::EchoApp;
#[cfg(feature = "kvstore-app")]
pub use application::kvstore::{KeyValueStoreApp, KeyValueStoreDriver};
pub use application::{Application, RequestDispatcher};
#[cfg(feature = "client")]
pub use client::{Client, ClientBuilder};
pub use error::Error;
