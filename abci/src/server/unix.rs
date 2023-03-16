//! ABCI application server interface.

use std::{fs, os::unix::net::UnixListener, path::Path};

use tracing::info;

use crate::{Error, RequestDispatcher};

/// A Unix socket-based server for serving a specific ABCI application.
///
/// Examples:
///
/// ```no_run
/// struct EchoApp {}
/// impl tenderdash_abci::Application for EchoApp{};
///
/// let socket = std::path::Path::new("/tmp/abci.sock");
/// let server = tenderdash_abci::start_unix(socket, EchoApp {}).expect("server failed");
///
/// loop {
///     match server.handle_connection() {
///         Ok(_) => {},
///         Err(e) => tracing::error!("error {}", e),
///     };
/// }
/// ```
pub struct UnixSocketServer<App: RequestDispatcher> {
    app: App,
    listener: UnixListener,
    read_buf_size: usize,
}

impl<App: RequestDispatcher> UnixSocketServer<App> {
    pub(super) fn bind(
        app: App,
        socket_file: &Path,
        read_buf_size: usize,
    ) -> Result<UnixSocketServer<App>, Error> {
        fs::remove_file(socket_file).ok();

        let listener = UnixListener::bind(socket_file)?;
        let socket_file = socket_file.to_path_buf();
        info!(
            "ABCI Unix server running at {:?}",
            socket_file.to_str().expect("wrong socket path")
        );

        let server = UnixSocketServer {
            app,
            listener,
            read_buf_size,
        };
        Ok(server)
    }

    /// Process one incoming connection.
    ///
    /// Returns when the connection is terminated or RequestDispatcher returns
    /// error.
    ///
    /// It is safe to call this method multiple times after it finishes;
    /// however, errors must be examined and handled, as the connection
    /// should not terminate.
    pub fn handle_connection(&self) -> Result<(), Error> {
        // let listener = self.listener;
        let stream = self.listener.accept()?;
        let name = String::from("<unix socket>");

        info!("Incoming Unix connection");

        super::handle_client(stream.0, name, &self.app, self.read_buf_size)
    }
}
