use std::path::Path;
use tenderdash_abci::{server::start_unix, Application};
use tracing::{span, Level};
use tracing_subscriber::filter::LevelFilter;

pub fn main() {
    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let span = span!(Level::TRACE, "my span");

    // Enter the span, returning a guard object.
    let _enter = span.enter();

    let socket = Path::new("/tmp/socket");
    let server = start_unix(socket, EchoApp {}).expect("server failed");
    loop {
        match server.handle_connection() {
            Ok(_) => {},
            Err(e) => tracing::error!("error {}", e),
        };
    }
}

/// Trivial echo application, mainly for testing purposes.
#[derive(Clone, Default)]
pub struct EchoApp;

impl Application for EchoApp {}
