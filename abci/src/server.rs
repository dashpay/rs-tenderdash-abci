//! Tenderdash ABCI Server.
mod codec;
pub mod tcp;
pub mod unix;

use std::{
    io::{Read, Write},
    net::ToSocketAddrs,
    path::Path,
};

use tracing::{error, info};
use unix::UnixSocketServer;

use crate::{
    application::RequestDispatcher,
    server::{codec::ServerCodec, tcp::TcpServer},
    Error,
};

/// The size of the read buffer for each incoming connection to the ABCI
/// server (1MB).
pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

/// Create new TCP server and bind to TCP address/port.
///
/// Use [`handle_connection()`] to accept connection and process all traffic in
/// this connection. Each incoming connection will be processed using `app`.
///
/// # Arguments
///
/// * `socket_file` - path to Unix socket file, for example:
///   `/var/run/abci.sock`
/// * `app` - request dispatcher, most likely implementation of Application
///   trait
///
///
/// # Return
///
/// Returns [`TcpServer`] which provides [`handle_connection()`] method. Call it
/// in a loop to accept and process incoming connections.
///
/// [`handle_connection()`]: unix::TcpServer::handle_connection()
///
/// # Examples
///
/// ```no_run
/// struct MyAbciApplication {};
/// impl tenderdash_abci::Application for MyAbciApplication {};
/// let app = MyAbciApplication {};
/// let addr =  std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(172, 17, 0, 1), 1234);
/// let server = tenderdash_abci::server::start_tcp(addr, app).expect("server failed");
/// loop {
///     server.handle_connection();
/// }
/// ```
pub fn start_tcp<App: RequestDispatcher>(
    addrs: impl ToSocketAddrs,
    app: App,
) -> Result<TcpServer<App>, Error> {
    TcpServer::bind(app, addrs)
}

/// start_unix creates new UnixSocketServer that binds to `socket_file`.
/// Use [`handle_connection()`] to accept connection and process all traffic in
/// this connection. Each incoming connection will be processed using `app`.
///
/// # Arguments
///
/// * `socket_file` - path to Unix socket file, for example:
///   `/var/run/abci.sock`
/// * `app` - request dispatcher, most likely implementation of Application
///   trait
///
///
/// # Return
///
/// Returns [`UnixSocketServer`] which provides [`handle_connection()`] method.
/// Call it in a loop to accept and process incoming connections.
///
/// [`handle_connection()`]: unix::UnixSocketServer::handle_connection()
///
/// # Examples
///
/// ```no_run
/// struct MyAbciApplication {};
/// impl tenderdash_abci::Application for MyAbciApplication {};
/// let app = MyAbciApplication {};
/// let socket = std::path::Path::new("/tmp/abci.sock");
/// let server = tenderdash_abci::server::start_unix(socket, app).expect("server failed");
/// loop {
///     server.handle_connection();
/// }
/// ```
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
                return Err(Error::connection_terminated());
            },
        };
        let response = app.handle(request)?;
        if let Err(e) = codec.send(response) {
            error!("Failed sending response to client {}: {:?}", name, e);
            return Err(e);
        }
    }
}
