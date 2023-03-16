use tenderdash_abci::{proto, start_server, Application, BindAddress};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

const SOCKET: &str = "/tmp/abci.sock";

pub fn main() {
    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Unix socket ABCI server example.");
    info!("This application listens on {SOCKET} and waits for incoming Tenderdash requests.");

    let socket = BindAddress::UnixSocket(SOCKET.to_string());
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
    fn echo(&self, request: proto::abci::RequestEcho) -> proto::abci::ResponseEcho {
        info!("received echo");
        proto::abci::ResponseEcho {
            message: request.message,
        }
    }
    /// Provide information about the ABCI application.
    fn info(&self, _request: proto::abci::RequestInfo) -> proto::abci::ResponseInfo {
        info!("received info request");
        proto::abci::ResponseInfo {
            app_version: 1,
            data: String::from("Echo Socket App"),
            version: String::from("1.0.0"),
            last_block_app_hash: Vec::from([0; 32]),
            last_block_height: 0,
        }
    }
}
