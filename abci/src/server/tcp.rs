//! ABCI application server interface.

use tokio::net::{TcpListener, ToSocketAddrs};
use tracing::info;

use super::{handle_client, Server, ServerRuntime, DEFAULT_SERVER_READ_BUF_SIZE};
use crate::{Error, RequestDispatcher, ServerCancel};

/// A TCP-based server for serving a specific ABCI application.
///
/// Only one incoming connection is handled at a time.
pub(super) struct TcpServer<App: RequestDispatcher> {
    app: App,
    listener: TcpListener,
    server_runtime: ServerRuntime,
    cancel: ServerCancel,
}

impl<App: RequestDispatcher> TcpServer<App> {
    pub(super) async fn bind<Addr>(
        app: App,
        addr: Addr,
        cancel: ServerCancel,
        server_runtime: ServerRuntime,
    ) -> Result<TcpServer<App>, Error>
    where
        Addr: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        info!(
            "ABCI TCP server  {} with proto {} running at {}",
            env!("CARGO_PKG_VERSION"),
            tenderdash_proto::ABCI_VERSION,
            local_addr
        );
        let server = TcpServer {
            app,
            listener,
            server_runtime,
            cancel,
        };
        Ok(server)
    }
}

impl<App: RequestDispatcher> Server for TcpServer<App> {
    fn handle_connection(&self) -> Result<(), Error> {
        tracing::trace!("handle connection");

        self.server_runtime.runtime_handle.block_on(async {
            let (stream, addr) = match self.listener.accept().await {
                Ok(conn) => conn,
                Err(e) => return Err(Error::Connection(e)),
            };
            let addr = addr.to_string();
            info!(addr, "incoming connection");

            handle_client(
                self.cancel.child_token(),
                stream,
                addr,
                &self.app,
                DEFAULT_SERVER_READ_BUF_SIZE,
            )
            .await
        })?;

        tracing::trace!("end of connection handling");
        Ok(())
    }
}

impl<App: RequestDispatcher> Drop for TcpServer<App> {
    fn drop(&mut self) {
        tracing::debug!(?self.listener, "ABCI tcp server shut down")
    }
}
