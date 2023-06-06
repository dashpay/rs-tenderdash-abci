//! ABCI application server interface.

use std::net::{TcpListener, ToSocketAddrs};

use tracing::info;

use super::{handle_client, Server, ServerCancel, DEFAULT_SERVER_READ_BUF_SIZE};
use crate::{Error, RequestDispatcher};

/// A TCP-based server for serving a specific ABCI application.
///
/// Only one incoming connection is handled at a time.
pub(super) struct TcpServer<App: RequestDispatcher> {
    app: App,
    listener: TcpListener,
    cancel: Box<dyn ServerCancel>,
}

impl<App: RequestDispatcher> TcpServer<App> {
    pub(super) fn bind<Addr>(
        cancel: Box<dyn ServerCancel>,
        app: App,
        addr: Addr,
    ) -> Result<TcpServer<App>, Error>
    where
        Addr: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr)?;
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
            cancel,
        };
        Ok(server)
    }
}

impl<App: RequestDispatcher> Server for TcpServer<App> {
    fn handle_connection(&self) -> Result<(), Error> {
        let (stream, addr) = self.listener.accept()?;
        let addr = addr.to_string();
        info!("Incoming connection from: {}", addr);

        handle_client(
            self.cancel.as_ref(),
            stream,
            addr,
            &self.app,
            DEFAULT_SERVER_READ_BUF_SIZE,
        )
    }
}

impl<App: RequestDispatcher> Drop for TcpServer<App> {
    fn drop(&mut self) {
        tracing::debug!(?self.listener, "ABCI tcp server shut down")
    }
}
