//! ABCI application server interface.

use std::net::{TcpListener, ToSocketAddrs};

use super::{handle_client, DEFAULT_SERVER_READ_BUF_SIZE};
use crate::{error::Error, Application};
use tracing::info;

/// A TCP-based server for serving a specific ABCI application.
///
/// Only one incoming connection is handled at a time.
pub struct TcpServer<App: Application> {
    app: App,
    listener: TcpListener,
}

impl<App: Application> TcpServer<App> {
    pub(super) fn bind<Addr>(app: App, addr: Addr) -> Result<TcpServer<App>, Error>
    where
        Addr: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).map_err(Error::io)?;
        let local_addr = listener.local_addr().map_err(Error::io)?.to_string();
        info!("ABCI server running at {}", local_addr);
        let server = TcpServer { app, listener };
        Ok(server)
    }

    /// Process one incoming connection. Returns once the connection is terminated.
    ///
    /// It is safe to call this method multiple times after it finishes;
    /// however, errors must be examined and handled, as the connection
    /// should not terminate.
    pub fn handle_connection(&self) -> Result<(), Error> {
        let (stream, addr) = self.listener.accept().map_err(Error::io)?;
        let addr = addr.to_string();
        info!("Incoming connection from: {}", addr);

        handle_client(stream, addr, &self.app, DEFAULT_SERVER_READ_BUF_SIZE)
    }
}
