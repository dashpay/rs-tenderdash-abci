use std::path::Path;
use tenderdash_abci::{server::start_unix, Application};
use tenderdash_proto::abci::{RequestEcho, RequestInfo, ResponseEcho, ResponseInfo};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

const SOCKET: &str = "/tmp/abci.sock";

pub fn main() {
    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Unix socket ABCI server example.");
    info!("This application listens on {SOCKET} and waits for incoming Tenderdash requests.");

    let socket = Path::new(SOCKET);
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

impl Application for EchoApp {
    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        info!("received echo");
        ResponseEcho {
            message: request.message,
        }
    }
    /// Provide information about the ABCI application.
    fn info(&self, _request: RequestInfo) -> ResponseInfo {
        info!("received info request");
        ResponseInfo {
            app_version: 1,
            data: String::from("Echo Socket App"),
            version: String::from("1.0.0"),
            last_block_app_hash: Vec::from([0; 32]),
            last_block_height: 0,
        }
    }
}
