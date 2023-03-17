//! ABCI application server interface.

use std::net::{TcpListener, ToSocketAddrs};

use tracing::info;

use super::{handle_client, Server, DEFAULT_SERVER_READ_BUF_SIZE};
use crate::{Error, RequestDispatcher};

/// A TCP-based server for serving a specific ABCI application.
///
/// Only one incoming connection is handled at a time.
pub(super) struct TcpServer<App: RequestDispatcher> {
    app: App,
    listener: TcpListener,
}

impl<App: RequestDispatcher> TcpServer<App> {
    pub(super) fn bind<Addr>(app: App, addr: Addr) -> Result<TcpServer<App>, Error>
    where
        Addr: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr)?;
        let local_addr = listener.local_addr()?;
        info!(
            "ABCI TCP server  {} with proto {} running at {}",
            env!("CARGO_PKG_VERSION"),
            tenderdash_proto::VERSION,
            local_addr
        );
        let server = TcpServer { app, listener };
        Ok(server)
    }
}

impl<App: RequestDispatcher> Server for TcpServer<App> {
    fn handle_connection(&self) -> Result<(), Error> {
        let (stream, addr) = self.listener.accept()?;
        let addr = addr.to_string();
        info!("Incoming connection from: {}", addr);

        handle_client(stream, addr, &self.app, DEFAULT_SERVER_READ_BUF_SIZE)
    }
}
