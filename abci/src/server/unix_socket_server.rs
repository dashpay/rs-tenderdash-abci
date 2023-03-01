//! ABCI application server interface.

use std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    thread,
};

use tracing::{error, info};

use crate::{application::RequestDispatcher, codec::ServerCodec, error::Error, Application};
use std::os::unix::net::{UnixListener, UnixStream};

/// A Unix socket-based server for serving a specific ABCI application.
///
/// Each incoming connection is handled in a separate thread. The ABCI
/// application is cloned for access in each thread. It is up to the
/// application developer to manage shared state across these different
/// threads.
pub struct UnixSocketServer<App> {
    app: App,
    listener: UnixListener,
    socket_filename: String,
    read_buf_size: usize,
}

impl<App: Application> UnixSocketServer<App> {
    /// Initiate a blocking listener for incoming connections.
    pub fn listen(self) -> Result<(), Error> {
        loop {
            let (stream, addr) = self.listener.accept().map_err(Error::io)?;
            let addr = addr.to_string();
            info!("Incoming connection from: {}", addr);
            self.spawn_client_handler(stream, addr);
        }
    }

    /// Getter for this server's local address.
    pub fn local_addr(&self) -> String {
        self.local_addr.clone()
    }

    fn spawn_client_handler(&self, stream: TcpStream, addr: String) {
        let app = self.app.clone();
        let read_buf_size = self.read_buf_size;
        let _ = thread::spawn(move || Self::handle_client(stream, addr, app, read_buf_size));
    }

    fn handle_client(stream: TcpStream, addr: String, app: App, read_buf_size: usize) {
        let mut codec = ServerCodec::new(stream, read_buf_size);
        info!("Listening for incoming requests from {}", addr);
        loop {
            let request = match codec.next() {
                Some(result) => match result {
                    Ok(r) => r,
                    Err(e) => {
                        error!(
                            "Failed to read incoming request from client {}: {:?}",
                            addr, e
                        );
                        return;
                    },
                },
                None => {
                    info!("Client {} terminated stream", addr);
                    return;
                },
            };
            let response = app.handle(request);
            if let Err(e) = codec.send(response) {
                error!("Failed sending response to client {}: {:?}", addr, e);
                return;
            }
        }
    }
}
