use tenderdash_abci::{proto::abci, start_server, Application};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

const SOCKET: &str = "/tmp/abci.sock";

pub fn main() {
    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Unix socket ABCI server example.");
    info!("This application listens on {SOCKET} and waits for incoming Tenderdash requests.");

    let socket = format!("unix://{}", SOCKET);
    let server = start_server(&socket, EchoApp {}).expect("server failed");
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

impl Application for EchoApp {
    fn echo(
        &self,
        request: abci::RequestEcho,
    ) -> Result<abci::ResponseEcho, abci::ResponseException> {
        info!("received echo");
        Ok(abci::ResponseEcho {
            message: request.message,
        })
    }
}
