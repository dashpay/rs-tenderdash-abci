//! Tenderdash ABCI Server.
mod codec;
#[cfg(feature = "tcp")]
mod tcp;
#[cfg(feature = "unix")]
mod unix;

use std::{
    net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6},
    str::FromStr,
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    runtime::{Handle, Runtime},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use url::Url;

#[cfg(feature = "tcp")]
use self::tcp::TcpServer;
#[cfg(feature = "unix")]
use self::unix::UnixSocketServer;
use crate::{application::RequestDispatcher, proto::abci, server::codec::Codec, Error};

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
    /// Returns when the connection is terminated, [cancel()] is called or
    /// RequestDispatcher returns `None`.
    ///
    /// It is safe to call this method multiple times after it finishes;
    /// however, errors must be examined and handled, as the connection
    /// should not terminate. One exception is [Error::Cancelled], which
    /// means server shutdown was requested.
    fn handle_connection(&self) -> Result<(), Error>;
}
pub type ServerCancel = CancellationToken;

/// Build new ABCI server and bind to provided address/port or socket.
///
/// Use [`handle_connection()`] to accept connection and process all traffic in
/// this connection. Each incoming connection will be processed using `app`.
///
/// # Examples
///
/// ```no_run
/// struct MyAbciApplication {};
/// impl tenderdash_abci::Application for MyAbciApplication {};
/// let app = MyAbciApplication {};
/// let bind_address = "unix:///tmp/abci.sock";
/// let server = tenderdash_abci::ServerBuilder::new(app, &bind_address).build().expect("server failed");
/// loop {
///     if let Err(tenderdash_abci::Error::Cancelled()) = server.handle_connection() {
///         break;
///     }
/// }
/// ```
///
/// [`handle_connection()`]: Server::handle_connection()
pub struct ServerBuilder<D>
where
    D: RequestDispatcher,
{
    app: D,
    bind_address: String,
    cancel: Option<ServerCancel>,
}

impl<'a, App: RequestDispatcher + 'a> ServerBuilder<App> {
    /// Create new server builder.
    ///
    /// # Arguments
    ///
    /// * `address` - address in URI format, pointing either to TCP address and
    ///   port (eg. `tcp://0.0.0.0:1234`, `tcp://[::1]:1234`) or Unix socket
    ///   (`unix:///var/run/abci.sock`)
    /// * `app` - request dispatcher, most likely implementation of Application
    ///   trait

    pub fn new(app: App, address: &str) -> Self {
        Self {
            app,
            bind_address: address.to_string(),
            cancel: None,
        }
    }

    /// Build the server and start listening.
    ///
    /// # Return
    ///
    /// Returns [`Server`] which provides [`Server::handle_connection()`]
    /// method. Call it in a loop to accept and process incoming
    /// connections.
    pub fn build(self) -> Result<Box<dyn Server + 'a>, crate::Error> {
        let app_address = url::Url::parse(self.bind_address.as_ref()).expect("invalid app address");
        if app_address.scheme() != "tcp" && app_address.scheme() != "unix" {
            panic!("app_address must be either tcp:// or unix://");
        }
        let server_runtime: ServerRuntime = ServerRuntime::default();

        let _guard = server_runtime.runtime_handle.enter();
        let hdl = server_runtime.runtime_handle.clone();
        hdl.block_on(self.start(server_runtime, app_address))
    }

    async fn start(
        self,
        rt: ServerRuntime,
        listen_address: Url,
    ) -> Result<Box<dyn Server + 'a>, Error> {
        // If no cancel is defined, so we add some "mock"
        let cancel = self.cancel.unwrap_or(ServerCancel::new());

        let server = match listen_address.scheme() {
            #[cfg(feature = "tcp")]
            "tcp" => Box::new(
                TcpServer::bind(self.app, parse_tcp_uri(listen_address), cancel, rt).await?,
            ) as Box<dyn Server + 'a>,
            #[cfg(feature = "unix")]
            "unix" => Box::new(UnixSocketServer::bind(
                self.app,
                listen_address.path(),
                cancel,
                DEFAULT_SERVER_READ_BUF_SIZE,
                rt,
            )?) as Box<dyn Server + 'a>,
            _ => panic!(
                "listen address uses unsupported scheme `{}`",
                listen_address.scheme()
            ),
        };

        Ok(server)
    }
    /// Set a [ServerCancel] token to support graceful shutdown.
    ///
    /// When [ServerCancel::is_cancelled()] returns `true`, server will
    /// stop gracefully after serving current request.
    ///
    /// [ServerCancel] is implemented by:
    ///
    /// * [AtomicBool], where `true` means server should shutdown,
    /// * [CancellationToken] when `tokio` feature is enabled.
    ///
    /// [CancellationToken]: tokio_util::sync::CancellationToken
    pub fn with_cancel_token(self, cancel: ServerCancel) -> Self {
        Self {
            cancel: Some(cancel),
            ..self
        }
    }
}

/// Server runtime that must be alive for the whole lifespan of the server
pub(crate) struct ServerRuntime {
    /// Runtime stored here to ensure it is never dropped
    _runtime: Option<Runtime>,
    runtime_handle: Handle,
}

impl Default for ServerRuntime {
    /// Return default server runtime.
    ///
    /// If tokio runtime is already initialized and entered, returns handle to
    /// it. Otherwise, creates new runtime and returns handle AND the
    /// runtime itself.
    fn default() -> Self {
        match Handle::try_current() {
            Ok(runtime_handle) => Self {
                runtime_handle,
                _runtime: None,
            },
            Err(_) => {
                let rt = Runtime::new().unwrap();
                Self {
                    runtime_handle: rt.handle().clone(),
                    _runtime: Some(rt),
                }
            },
        }
    }
}

#[deprecated = "use `ServerBuilder::new(app, &bind_address).build()` instead"]
pub fn start_server<'a, App: RequestDispatcher + 'a, Addr>(
    bind_address: Addr,
    app: App,
) -> Result<Box<dyn Server + 'a>, crate::Error>
where
    Addr: AsRef<str>,
{
    ServerBuilder::new(app, bind_address.as_ref()).build()
}

/// handle_client accepts one client connection and handles received messages.
pub(crate) async fn handle_client<App, S>(
    cancel_token: ServerCancel,
    stream: S,
    name: String,
    app: &App,
    read_buf_size: usize,
) -> Result<(), Error>
where
    App: RequestDispatcher,
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut codec = Codec::new(stream, read_buf_size, cancel_token);
    info!("Listening for incoming requests from {}", name);

    loop {
        let Some(request) = codec.receive().await? else {
            error!("Client {} terminated stream", name);
            return Ok(())
        };

        let Some(response) = app.handle(request.clone())  else {
            // `RequestDispatcher` decided to stop receiving new requests:
            info!("ABCI Application is shutting down");
            return Ok(());
        };

        if let Some(abci::response::Value::Exception(ex)) = response.value.clone() {
            error!(error = ex.error, ?request, "error processing request")
        }

        codec.send(response).await?;
    }
}

fn parse_tcp_uri(uri: url::Url) -> SocketAddr {
    let host = uri.host_str().unwrap();
    // remove '[' and ']' from ipv6 address, as per https://github.com/servo/rust-url/issues/770
    let host = host.replace(['[', ']'], "");
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
