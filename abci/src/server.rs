//! Tenderdash ABCI Server.
mod codec;
pub mod tcp;
pub mod unix;

use crate::{
    application::RequestDispatcher, server::codec::ServerCodec, server::tcp::TcpServer,
    Application, Error,
};
use std::{
    io::{Read, Write},
    net::ToSocketAddrs,
    path::Path,
};
use tracing::{error, info};

use unix::UnixSocketServer;

/// The size of the read buffer for each incoming connection to the ABCI
/// server (1MB).
pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

/// start_tcp creates a server that listens on `addresses`.
/// Each incoming connection will be processed using `app`.
/// Use [`handle_connection()`] to accept connection and process all traffic in this connection.
///
/// # Example
/// ```
/// let server = start_tcp(addresses, app)
/// loop {
///    let result = server.handle_connection()
///    // handle result errors
/// }
///
/// [`handle_connection()`]: unix::UnixSocketServer::handle_connection()
pub fn start_tcp<App: Application>(
    addrs: impl ToSocketAddrs,
    app: App,
) -> Result<TcpServer<App>, Error> {
    TcpServer::bind(app, addrs)
}

/// start_unix creates new UnixSocketServer that binds to `socket_file`.
/// Each incoming connection will be processed using `app`.
/// Use [`handle_connection()`] to accept connection and process all traffic in this connection.
///
/// [`handle_connection()`]: unix::UnixSocketServer::handle_connection()
pub fn start_unix<App: RequestDispatcher>(
    socket_file: &Path,
    app: App,
) -> Result<UnixSocketServer<App>, Error> {
    info!(
        "starting unix server on socket file {}",
        socket_file.to_str().expect("invalid socket file")
    );
    UnixSocketServer::bind(app, socket_file, DEFAULT_SERVER_READ_BUF_SIZE)
}

/// handle_client accepts one client connection and handles received messages.
pub(crate) fn handle_client<App, S>(
    stream: S,
    name: String,
    app: &App,
    read_buf_size: usize,
) -> Result<(), Error>
where
    App: RequestDispatcher,
    S: Read + Write,
{
    let mut codec = ServerCodec::new(stream, read_buf_size);
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
                    return Err(e);
                },
            },
            None => {
                info!("Client {} terminated stream", name);
                return Err(Error::server_connection_terminated());
            },
        };
        let response = app.handle(request)?;
        if let Err(e) = codec.send(response) {
            error!("Failed sending response to client {}: {:?}", name, e);
            return Err(e);
        }
    }
}
