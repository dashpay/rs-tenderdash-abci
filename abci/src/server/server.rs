use crate::{
    application::RequestDispatcher, codec::ServerCodec, error::Error, server::tcp::TcpServer,
    Application,
};
use std::{net::ToSocketAddrs, path::Path};
use tracing::{error, info};

use super::unix::UnixSocketServer;

/// The size of the read buffer for each incoming connection to the ABCI
/// server (1MB).
pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

// start_tcp_server listens on `addresses` and processes incoming connections.
// Each incoming connection will be processed in a separate thread, using clone of `app`.
// All received requests are dispatched to the `app`.
// It blocks indefinitely.
pub fn start_tcp_server<App: Application>(
    addresses: impl ToSocketAddrs,
    app: App,
) -> Result<(), Error> {
    let srv = TcpServer::bind(app, addresses)?;
    srv.listen()
}

// start_unix_server connects to `socket_file` and processes incoming connections.
// Each incoming connection will be processed in a separate thread, using clone of `app`.
// All received requests are dispatched to the `app`.
//
// It blocks indefinitely.
pub fn start_unix_server<App: Application>(socket_file: &Path, app: App) -> Result<(), Error> {
    info!("starting unix server on socket file {}", socket_file.to_str().expect("invalid socket file"));
    let srv = UnixSocketServer::bind(app, socket_file, DEFAULT_SERVER_READ_BUF_SIZE)?;
    srv.listen()
}

pub(crate) trait ReadWriter: std::io::Read + std::io::Write + Send + Sync + 'static {
    // fn clone(&self) -> Self ;
    fn clone(&self) -> Self;
}

pub(crate) struct ClientThread<App: RequestDispatcher, S: ReadWriter> {
    stream: S,
    app: App,
    name: String,
    read_buf_size: usize,
}

impl<App: RequestDispatcher, S: ReadWriter> ClientThread<App, S> {
    pub(crate) fn new(s: S, name: String, app: App, read_buf_size: usize) -> Self {
        ClientThread {
            stream: s,
            app,
            name,
            read_buf_size,
        }
    }

    pub(crate) fn handle_client(thread: ClientThread<App, S>) {
        let stream = thread.stream;
        let name = thread.name;
        let app = thread.app;

        let mut codec = ServerCodec::new(stream, thread.read_buf_size);
        info!("Listening for incoming requests from {}", name);
        loop {
            let request = match codec.next() {
                Some(result) => match result {
                    Ok(r) => r,
                    Err(e) => {
                        error!(
                            "Failed to read incoming request from client {}: {:?}",
                            name, e
                        );
                        return;
                    },
                },
                None => {
                    info!("Client {} terminated stream", name);
                    return;
                },
            };
            let response = app.handle(request);
            if let Err(e) = codec.send(response) {
                error!("Failed sending response to client {}: {:?}", name, e);
                return;
            }
        }
    }
}
