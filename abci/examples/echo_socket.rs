use lazy_static::lazy_static;
use tenderdash_abci::{proto::abci, Application, CancellationToken, ServerBuilder};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

const SOCKET: &str = "/tmp/abci.sock";
lazy_static! {
    static ref CANCEL_TOKEN: CancellationToken = CancellationToken::new();
}
pub fn main() {
    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Unix socket ABCI server example.");
    info!("This application listens on {SOCKET} and waits for incoming Tenderdash requests.");

    let socket = format!("unix://{}", SOCKET);
    let app = EchoApp {};

    let cancel = CANCEL_TOKEN.clone();
    let mut server = ServerBuilder::new(app, &socket)
        .with_cancel_token(cancel)
        .build()
        .expect("server failed");

    loop {
        match server.next_client() {
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
        info!("received echo, cancelling");

        let cancel = CANCEL_TOKEN.clone();
        cancel.cancel();

        Ok(abci::ResponseEcho {
            message: request.message,
        })
    }
}
