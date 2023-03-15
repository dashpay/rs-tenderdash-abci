//! Tenderdash ABCI Server.
mod codec;
mod tcp;
mod unix;

use std::{
    io::{Read, Write},
    net::ToSocketAddrs,
    path::Path,
};

use tracing::info;
use unix::UnixSocketServer;

use crate::{
    application::RequestDispatcher,
    server::{codec::Codec, tcp::TcpServer},
    Error,
};

/// The size of the read buffer for each incoming connection to the ABCI
/// server (1MB).
pub(crate) const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

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
/// let server = tenderdash_abci::start_tcp(addr, app).expect("server failed");
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
/// let server = tenderdash_abci::start_unix(socket, app).expect("server failed");
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
    let mut codec = Codec::new(stream, read_buf_size);
    info!("Listening for incoming requests from {}", name);

    loop {
        let Some(request) = codec.receive()? else {
            info!("Client {} terminated stream", name);
            return Ok(())
        };
        let Some(response) = app.handle(request)? else {
            // `RequestDispatcher` decided to stop receiving new requests:
            info!("ABCI Application is shutting down");
            return Ok(());
        };

        codec.send(response)?;
    }
}
