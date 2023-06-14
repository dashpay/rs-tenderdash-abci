//! ABCI application server interface.

use std::sync::Arc;

use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::Mutex,
};
use tracing::info;

use super::{handle_client, Server, ServerRuntime};
use crate::{Error, RequestDispatcher, ServerCancel};

/// A TCP-based server for serving a specific ABCI application.
///
/// Only one incoming connection is handled at a time.
pub(super) struct TcpServer<App: RequestDispatcher> {
    app: App,
    listener: Arc<Mutex<TcpListener>>,
    server_runtime: ServerRuntime,
    cancel: ServerCancel,
}

impl<App: RequestDispatcher> TcpServer<App> {
    pub(super) fn bind<Addr>(
        app: App,
        addr: Addr,
        cancel: ServerCancel,
        server_runtime: ServerRuntime,
    ) -> Result<TcpServer<App>, Error>
    where
        Addr: ToSocketAddrs + Send + 'static,
    {
        let listener = server_runtime.call_async(TcpListener::bind(addr), cancel.clone())??;

        // let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        info!(
            "ABCI TCP server  {} with proto {} running at {}",
            env!("CARGO_PKG_VERSION"),
            tenderdash_proto::ABCI_VERSION,
            local_addr
        );
        let server = TcpServer::<App> {
            app,
            listener: Arc::new(Mutex::new(listener)),
            server_runtime,
            cancel,
        };
        Ok(server)
    }
}

impl<'a, App: RequestDispatcher + 'a> Server for TcpServer<App> {
    fn next_client(&self) -> Result<(), Error> {
        // create child token to cancel this connection on error, but not the caller
        let cancel = self.cancel.child_token();
        handle_client(
            cancel.clone(),
            &self.listener,
            &self.app,
            &self.server_runtime,
        )?;

        tracing::trace!("end of connection");
        cancel.cancel();

        Ok(())
    }
}

impl<App: RequestDispatcher> Drop for TcpServer<App> {
    fn drop(&mut self) {
        tracing::debug!(?self.listener, "ABCI tcp server shut down")
    }
}
