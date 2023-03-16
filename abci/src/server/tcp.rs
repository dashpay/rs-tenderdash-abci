//! ABCI application server interface.

use std::net::{TcpListener, ToSocketAddrs};

use tracing::info;

use super::{handle_client, DEFAULT_SERVER_READ_BUF_SIZE};
use crate::{Error, RequestDispatcher};

/// A TCP-based server for serving a specific ABCI application.
///
/// Only one incoming connection is handled at a time.
pub struct TcpServer<App: RequestDispatcher> {
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
        info!("ABCI server running at {}", local_addr);
        let server = TcpServer { app, listener };
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
        let (stream, addr) = self.listener.accept()?;
        let addr = addr.to_string();
        info!("Incoming connection from: {}", addr);

        handle_client(stream, addr, &self.app, DEFAULT_SERVER_READ_BUF_SIZE)
    }
}
