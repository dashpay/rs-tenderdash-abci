//! ABCI application server interface.

use std::{fs, os::unix::net::UnixListener, path::Path};

use tracing::info;

use super::Server;
use crate::{Error, RequestDispatcher};

/// A Unix socket-based server for serving a specific ABCI application.
pub(super) struct UnixSocketServer<App: RequestDispatcher> {
    app: App,
    listener: UnixListener,
    read_buf_size: usize,
}

impl<App: RequestDispatcher> UnixSocketServer<App> {
    pub(super) fn bind<P>(
        app: App,
        socket_file: P,
        read_buf_size: usize,
    ) -> Result<UnixSocketServer<App>, Error>
    where
        P: AsRef<Path>,
    {
        let socket_file = socket_file.as_ref();
        fs::remove_file(socket_file).ok();

        let listener = UnixListener::bind(socket_file)?;
        info!(
            "ABCI Unix server {} with proto {} running at {:?}",
            env!("CARGO_PKG_VERSION"),
            tenderdash_proto::TENDERDASH_VERSION,
            socket_file.to_str().expect("wrong socket path")
        );

        let server = UnixSocketServer {
            app,
            listener,
            read_buf_size,
        };
        Ok(server)
    }
}

impl<App: RequestDispatcher> Server for UnixSocketServer<App> {
    fn handle_connection(&self) -> Result<(), Error> {
        // let listener = self.listener;
        let stream = self.listener.accept()?;
        let name = String::from("<unix socket>");

        info!("Incoming Unix connection");

        super::handle_client(stream.0, name, &self.app, self.read_buf_size)
    }
}
