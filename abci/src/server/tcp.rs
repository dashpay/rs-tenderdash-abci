//! ABCI application server interface.

use std::{
    net::{ TcpListener, TcpStream, ToSocketAddrs},
    thread,
};

use tracing::{ info};

use crate::{
     error::Error,
    server::server::DEFAULT_SERVER_READ_BUF_SIZE, Application,
};

use super::server::{ClientThread, ReadWriter};

/// A TCP-based server for serving a specific ABCI application.
///
/// Each incoming connection is handled in a separate thread. The ABCI
/// application is cloned for access in each thread. It is up to the
/// application developer to manage shared state across these different
/// threads.
pub struct TcpServer<App: Application> {
    app: App,
    listener: TcpListener,
}

impl<App: Application> TcpServer<App> {
    pub(in crate::server) fn bind<Addr>(app: App, addr: Addr) -> Result<TcpServer<App>, Error>
    where
        Addr: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).map_err(Error::io)?;
        let local_addr = listener.local_addr().map_err(Error::io)?.to_string();
        info!("ABCI server running at {}", local_addr);
        let server = TcpServer {
            app,
            listener,
                    };
        Ok(server)
    }

    /// Initiate a blocking listener for incoming connections.
    pub(in crate::server) fn listen(self) -> Result<(), Error> {
        loop {
            let ( stream, addr) = self.listener.accept().map_err(Error::io)?;
            let addr = addr.to_string();
            info!("Incoming connection from: {}", addr);

            let app = self.app.clone();
            let thread = ClientThread::new(stream, addr, app, DEFAULT_SERVER_READ_BUF_SIZE);
            thread::spawn(move || ClientThread::handle_client(thread));
        }
    }
}

impl ReadWriter for TcpStream {
    fn clone(&self) -> Self {
        self.try_clone().expect("cannot clone TcpStream")
    }
}
