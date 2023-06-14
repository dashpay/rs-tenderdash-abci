//! ABCI application server interface.

use std::{fs, path::Path, sync::Arc};

use tokio::{net::UnixListener, sync::Mutex};
use tracing::info;

use super::{Server, ServerCancel, ServerRuntime};
use crate::{Error, RequestDispatcher};

/// A Unix socket-based server for serving a specific ABCI application.
pub(super) struct UnixSocketServer<App: RequestDispatcher> {
    app: App,
    listener: Arc<Mutex<UnixListener>>,
    cancel: ServerCancel,
    server_runtime: ServerRuntime,
}

impl<App: RequestDispatcher> UnixSocketServer<App> {
    pub(super) fn bind<P>(
        app: App,
        socket_file: P,
        cancel: ServerCancel,
        server_runtime: ServerRuntime,
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
            tenderdash_proto::ABCI_VERSION,
            socket_file.to_str().expect("wrong socket path")
        );

        let server = UnixSocketServer {
            app,
            listener: Arc::new(Mutex::new(listener)),
            cancel,
            server_runtime,
        };
        Ok(server)
    }
}

impl<App: RequestDispatcher> Server for UnixSocketServer<App> {
    fn next_client(&self) -> Result<(), Error> {
        info!("Incoming Unix connection");
        super::handle_client(
            self.cancel.clone(),
            &self.listener,
            &self.app,
            &self.server_runtime,
        )
    }
}

impl<App: RequestDispatcher> Drop for UnixSocketServer<App> {
    fn drop(&mut self) {
        tracing::debug!(?self.listener, "ABCI unix socket server shut down")
    }
}
