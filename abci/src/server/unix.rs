//! ABCI application server interface.

use std::{fs::remove_file, path::Path};

use crate::{server::server::handle_client, Application, Error};
use std::os::unix::net::UnixListener;
use tracing::info;

/// A Unix socket-based server for serving a specific ABCI application.
///
/// Example:
/// ```
/// let socket = Path::new("/tmp/socket");
/// let server = start_unix(socket, EchoApp {}).expect("server failed");
/// loop {
///     match server.handle_connection() {
///         Ok(_) => {},
///         Err(e) => tracing::error!("error {}", e),
///     };
/// }
/// ```
pub struct UnixSocketServer<App: Application> {
    app: App,
    listener: UnixListener,
    read_buf_size: usize,
}

impl<App: Application> UnixSocketServer<App> {
    pub(super) fn bind(
        app: App,
        socket_file: &Path,
        read_buf_size: usize,
    ) -> Result<UnixSocketServer<App>, Error> {
        _ = remove_file(socket_file);

        let listener = UnixListener::bind(socket_file).map_err(Error::io)?;
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
    /// Returns once the connection is terminated.
    ///
    /// It is safe to call this method multiple times after it finishes;
    /// however, errors must be examined and handled, as the connection
    /// should not terminate.
    pub fn handle_connection(&self) -> Result<(), Error> {
        // let listener = self.listener;
        let stream = self.listener.accept().map_err(Error::io)?;
        let name = String::from("<unix socket>");

        info!("Incoming Unix connection");

        handle_client(stream.0, name, &self.app, self.read_buf_size)
    }
}
