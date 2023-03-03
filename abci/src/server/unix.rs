//! ABCI application server interface.

use std::path::Path;

use crate::{server::server::handle_client, Application, Error};
use std::os::unix::net::{UnixListener, UnixStream};
use tracing::info;

use super::server::ReadWriter;

/// A Unix socket-based server for serving a specific ABCI application.
///
/// Each incoming connection is handled in a separate thread. The ABCI
/// application is cloned for access in each thread. It is up to the
/// application developer to manage shared state across these different
/// threads.
pub struct UnixSocketServer<App: Application> {
    app: App,
    listener: UnixListener,
    read_buf_size: usize,
}

impl<App: Application> UnixSocketServer<App> {
    pub(in crate::server) fn bind(
        app: App,
        socket_file: &Path,
        read_buf_size: usize,
    ) -> Result<UnixSocketServer<App>, Error> {
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
    /// Initiate a blocking listener for incoming connections.
    pub fn handle_connection(self) -> Result<(), Error> {
        // let listener = self.listener;
        let stream = self.listener.accept().map_err(Error::io)?;
        info!("Incoming Unix connection");

        handle_client(
            stream.0,
            String::from("Unix"),
            self.app.clone(),
            self.read_buf_size,
        )
    }
}

impl ReadWriter for UnixStream {}
