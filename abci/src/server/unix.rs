//! ABCI application server interface.

use std::{fs, path::Path};

use tokio::net::UnixListener;
use tracing::info;

use super::{Server, ServerCancel, ServerRuntime};
use crate::{Error, RequestDispatcher};

/// A Unix socket-based server for serving a specific ABCI application.
pub(super) struct UnixSocketServer<App: RequestDispatcher> {
    app: App,
    listener: UnixListener,
    read_buf_size: usize,
    cancel: ServerCancel,
    server_runtime: ServerRuntime,
}

impl<App: RequestDispatcher> UnixSocketServer<App> {
    pub(super) fn bind<P>(
        app: App,
        socket_file: P,
        cancel: ServerCancel,
        read_buf_size: usize,
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
            listener,
            read_buf_size,
            cancel,
            server_runtime,
        };
        Ok(server)
    }
}

impl<App: RequestDispatcher> Server for UnixSocketServer<App> {
    fn handle_connection(&self) -> Result<(), Error> {
        // let listener = self.listener;
        let name = String::from("<unix socket>");

        info!("Incoming Unix connection");

        self.server_runtime.runtime_handle.block_on(async {
            let stream = self.listener.accept().await?;

            super::handle_client(
                self.cancel.child_token(),
                stream.0,
                name,
                &self.app,
                self.read_buf_size,
            )
            .await
        })
    }
}

impl<App: RequestDispatcher> Drop for UnixSocketServer<App> {
    fn drop(&mut self) {
        tracing::debug!(?self.listener, "ABCI unix socket server shut down")
    }
}
