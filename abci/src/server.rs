//! Tenderdash ABCI Server.
mod codec;
mod tcp;
mod unix;

use core::fmt;
use std::{
    fmt::Display,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use serde::{Deserialize, Serialize};
use tracing::info;

use self::{tcp::TcpServer, unix::UnixSocketServer};
use crate::{application::RequestDispatcher, server::codec::Codec, Error};

/// The size of the read buffer for each incoming connection to the ABCI
/// server (1MB).
pub(crate) const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

/// ABCI Server handle.
///
/// Use [`Server::handle_connection()`] to accept connection and process all
/// traffic in this connection. Each incoming connection will be processed using
/// `app`.
pub trait Server {
    /// Process one incoming connection.
    ///
    /// Returns when the connection is terminated or RequestDispatcher returns
    /// error.
    ///
    /// It is safe to call this method multiple times after it finishes;
    /// however, errors must be examined and handled, as the connection
    /// should not terminate.
    fn handle_connection(&self) -> Result<(), Error>;
}

/// Address to listen on, either TCP address or Unix Socket path
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BindAddress {
    UnixSocket(String),
    TCP(SocketAddr),
}

impl Default for BindAddress {
    fn default() -> Self {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6783);
        BindAddress::TCP(SocketAddr::V4(addr))
    }
}
impl Display for BindAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindAddress::UnixSocket(s) => write!(f, "unix://{s}"),
            BindAddress::TCP(s) => write!(f, "tcp://").and_then(|_| s.fmt(f)),
        }
    }
}

/// Create new ABCI server and bind to provided address/port or socket.
///
/// Use [`handle_connection()`] to accept connection and process all traffic in
/// this connection. Each incoming connection will be processed using `app`.
///
/// # Arguments
///
/// * `address` - a [BindAddress], pointing either to TCP address and port or
///   Unix socket
/// * `app` - request dispatcher, most likely implementation of Application
///   trait
///
///
/// # Return
///
/// Returns [`Server`] which provides [`handle_connection()`] method. Call it
/// in a loop to accept and process incoming connections.
///
/// [`handle_connection()`]: Server::handle_connection()
///
/// # Examples
///
/// ```no_run
/// struct MyAbciApplication {};
/// impl tenderdash_abci::Application for MyAbciApplication {};
/// let app = MyAbciApplication {};
/// let bind_address = tenderdash_abci::BindAddress::UnixSocket("/tmp/abci.sock".to_string());
/// let server = tenderdash_abci::start_server(&bind_address, app).expect("server failed");
/// loop {
///     server.handle_connection();
/// }
/// ```
pub fn start_server<'a, App: RequestDispatcher + 'a>(
    bind_address: &BindAddress,
    app: App,
) -> Result<Box<dyn Server + 'a>, crate::Error> {
    let server = match bind_address {
        BindAddress::TCP(addr) => Box::new(TcpServer::bind(app, addr)?) as Box<dyn Server + 'a>,

        BindAddress::UnixSocket(socket_file) => Box::new(UnixSocketServer::bind(
            app,
            socket_file.as_ref(),
            DEFAULT_SERVER_READ_BUF_SIZE,
        )?) as Box<dyn Server + 'a>,
    };

    Ok(server)
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
