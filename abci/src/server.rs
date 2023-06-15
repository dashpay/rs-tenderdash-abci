//! Tenderdash ABCI Server.
mod codec;
mod generic;

use std::{
    net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6},
    str::FromStr,
};

use tokio::{
    net::{TcpListener, UnixListener},
    runtime::{Handle, Runtime},
};

use self::generic::GenericServer;
use crate::{application::RequestDispatcher, Error};

/// ABCI Server handle.
///
/// Use [`Server::handle_connection()`] to accept connection and process all
/// traffic in this connection. Each incoming connection will be processed using
/// `app`.
pub trait Server {
    /// Process one incoming connection.
    ///
    /// Returns when the connection is terminated, [CancellationToken::cancel()]
    /// is called or RequestDispatcher returns `None`.
    ///
    /// It is safe to call this method multiple times after it finishes;
    /// however, errors must be examined and handled, as the connection
    /// should not terminate. One exception is [Error::Cancelled], which
    /// means server shutdown was requested.
    fn next_client(&self) -> Result<(), Error>;

    #[deprecated = "use `next_client()`"]
    fn handle_connection(&self) -> Result<(), Error> {
        self.next_client()
    }
}

pub type CancellationToken = tokio_util::sync::CancellationToken;

/// ABCI server builder that creates and starts ABCI server
///
/// Create new server with [`ServerBuilder::new()`], configure it as needed, and
/// finalize using [`ServerBuilder::build()`]. This will create and start new
/// ABCI server.
///
/// Use [`Server::next_client()`] to accept connection from ABCI client
/// (Tenderdash) and start processing incoming requests. Each incoming
/// connection will be processed using `app`.
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
///     if let Err(tenderdash_abci::Error::Cancelled()) = server.next_client() {
///         break;
///     }
/// }
/// ```
pub struct ServerBuilder<D>
where
    D: RequestDispatcher,
{
    app: D,
    bind_address: String,
    cancel: Option<CancellationToken>,
    server_runtime: Option<ServerRuntime>,
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
            server_runtime: None,
        }
    }

    /// Build and start the ABCI server.
    ///
    /// # Return
    ///
    /// Returns [`Server`] which provides [`Server::next_client()`]
    /// method. Call it in a loop to accept and process incoming
    /// connections.
    pub fn build(self) -> Result<Box<dyn Server + 'a>, crate::Error> {
        let bind_address =
            url::Url::parse(self.bind_address.as_ref()).expect("invalid bind address");
        if bind_address.scheme() != "tcp" && bind_address.scheme() != "unix" {
            panic!("app_address must be either tcp:// or unix://");
        }
        let server_runtime: ServerRuntime = self.server_runtime.unwrap_or_default();

        let _guard = server_runtime.handle.enter();

        // No cancel is defined, so we add some "mock"
        let cancel = self.cancel.unwrap_or(CancellationToken::new());

        let server = match bind_address.scheme() {
            #[cfg(feature = "tcp")]
            "tcp" => Box::new(GenericServer::<App, TcpListener>::bind(
                self.app,
                parse_tcp_uri(bind_address),
                cancel,
                server_runtime,
            )?) as Box<dyn Server + 'a>,
            #[cfg(feature = "unix")]
            "unix" => Box::new(GenericServer::<App, UnixListener>::bind(
                self.app,
                bind_address.path(),
                cancel,
                server_runtime,
            )?) as Box<dyn Server + 'a>,
            _ => panic!(
                "listen address uses unsupported scheme `{}`",
                bind_address.scheme()
            ),
        };

        Ok(server)
    }
    /// Set a [CancellationToken] token to support graceful shutdown.
    ///
    /// Call [`CancellationToken::cancel()`] to stop the server gracefully.
    ///
    /// [`CancellationToken::cancel()`]: tokio_util::sync::CancellationToken::cancel()
    pub fn with_cancel_token(self, cancel: CancellationToken) -> Self {
        Self {
            cancel: Some(cancel),
            ..self
        }
    }
    /// Set tokio [Runtime](tokio::runtime::Runtime) to use.
    ///
    /// By default, current tokio runtime is used. If no runtime is active
    /// ([Handle::try_current()] returns error), new multi-threaded runtime
    /// is started. If this is not what you want, use
    /// [ServerBuilder::with_runtime()] to provide handler to correct Tokio
    /// runtime.
    ///
    /// # Example
    ///
    /// ```
    /// use tokio::runtime::{Handle, Runtime};
    /// use tenderdash_abci::{RequestDispatcher, ServerBuilder, CancellationToken, Application};
    ///
    /// // Your custom RequestDispatcher implementation
    /// struct MyApp;
    ///
    /// impl Application for MyApp {}
    ///
    /// fn main() {
    ///     // Create a Tokio runtime
    ///     let runtime = Runtime::new().unwrap();
    ///     let runtime_handle = runtime.handle().clone();
    ///
    ///     // Create an instance of your RequestDispatcher implementation
    ///     let app = MyApp;
    ///
    ///     // Create cancellation token
    ///     let cancel = CancellationToken::new();
    ///     # cancel.cancel();
    ///     // Create a ServerBuilder instance and set the runtime using with_runtime()
    ///     
    ///     let server = ServerBuilder::new(app, "tcp://0.0.0.0:17534")
    ///         .with_runtime(runtime_handle)
    ///         .with_cancel_token(cancel)
    ///         .build();
    /// }
    /// ```
    ///
    /// In this example, we first create a Tokio runtime and get its handle.
    /// Then we create an instance of our `MyApp` struct that implements the
    /// `RequestDispatcher` trait. We create a `ServerBuilder` instance by
    /// calling `new()` with our `MyApp` instance and then use the
    /// `with_runtime()` method to set the runtime handle. Finally, you can
    /// continue building your server and eventually run it.
    ///
    /// [Handle::try_current()]: tokio::runtime::Handle::try_current()
    pub fn with_runtime(self, runtime_handle: Handle) -> Self {
        Self {
            server_runtime: Some(ServerRuntime {
                _runtime: None,
                handle: runtime_handle,
            }),
            ..self
        }
    }
}

/// Server runtime that must be alive for the whole lifespan of the server
pub(crate) struct ServerRuntime {
    /// Runtime stored here to ensure it is never dropped
    _runtime: Option<Runtime>,
    handle: Handle,
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
                handle: runtime_handle,
                _runtime: None,
            },
            Err(_) => {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(4)
                    .enable_all()
                    .build()
                    .expect("cannot create runtime");
                tracing::trace!("created new runtime");
                Self {
                    handle: rt.handle().clone(),
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
