//! Tenderdash ABCI Server.
mod codec;
mod tcp;
mod unix;

use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6},
    str::FromStr,
};

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

/// Create new ABCI server and bind to provided address/port or socket.
///
/// Use [`handle_connection()`] to accept connection and process all traffic in
/// this connection. Each incoming connection will be processed using `app`.
///
/// # Arguments
///
/// * `address` - address in URI format, pointing either to TCP address and port
///   (eg. `tcp://0.0.0.0:1234`, `tcp://[::1]:1234`) or Unix socket
///   (`unix:///var/run/abci.sock`)
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
/// let bind_address = "unix:///tmp/abci.sock";
/// let server = tenderdash_abci::start_server(&bind_address, app).expect("server failed");
/// loop {
///     server.handle_connection();
/// }
/// ```
pub fn start_server<'a, App: RequestDispatcher + 'a, Addr>(
    bind_address: Addr,
    app: App,
) -> Result<Box<dyn Server + 'a>, crate::Error>
where
    Addr: AsRef<str>,
{
    let app_address = url::Url::parse(bind_address.as_ref()).expect("invalid app address");
    if app_address.scheme() != "tcp" && app_address.scheme() != "unix" {
        panic!("app_address must be either tcp:// or unix://");
    }

    let server = match app_address.scheme() {
        "tcp" => {
            Box::new(TcpServer::bind(app, parse_tcp_uri(app_address))?) as Box<dyn Server + 'a>
        },
        "unix" => Box::new(UnixSocketServer::bind(
            app,
            app_address.path(),
            DEFAULT_SERVER_READ_BUF_SIZE,
        )?) as Box<dyn Server + 'a>,
        _ => panic!(
            "listen address uses unsupported scheme `{}`",
            app_address.scheme()
        ),
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

fn parse_tcp_uri(uri: url::Url) -> SocketAddr {
    let host = uri.host_str().unwrap();
    // remove '[' and ']' from ipv6 address, as per https://github.com/servo/rust-url/issues/770
    let host = host.replace('[', "").replace(']', "");
    let port = uri.port().expect("missing tcp port");

    let ip = IpAddr::from_str(host.as_str())
        .unwrap_or_else(|e| panic!("invalid listen address {}: {}", host, e));
    match ip {
        IpAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(a, port)),
        IpAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(a, port, 0, 0)),
    }
}

#[cfg(test)]
mod tests {
    use crate::server::parse_tcp_uri;

    #[test]
    fn test_parse_tcp_uri() {
        struct TestCase<'a> {
            uri: &'a str,
            expect: &'a str,
        }

        let test_cases = [
            TestCase {
                uri: "tcp://0.0.0.0:1234",
                expect: "0.0.0.0:1234",
            },
            TestCase {
                uri: "tcp://[::]:1234",
                expect: "[::]:1234",
            },
            TestCase {
                uri: "tcp://[::1]:1234",
                expect: "[::1]:1234",
            },
            TestCase {
                uri: "tcp://[::ffff:ac11:1]:5678",
                expect: "[::ffff:172.17.0.1]:5678",
            },
        ];

        for test_case in test_cases {
            let uri = url::Url::parse(test_case.uri).unwrap();

            let addr = parse_tcp_uri(uri);
            assert_eq!(test_case.expect, addr.to_string());
        }
    }
}
