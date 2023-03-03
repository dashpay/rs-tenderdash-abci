//! ABCI application server interface.

use std::{path::Path, thread};

use tracing::{error, info};

use crate::{error::Error, Application};
use std::os::unix::net::{UnixListener, UnixStream};

use super::server::{ClientThread, ReadWriter};

/// A Unix socket-based server for serving a specific ABCI application.
///
/// Each incoming connection is handled in a separate thread. The ABCI
/// application is cloned for access in each thread. It is up to the
/// application developer to manage shared state across these different
/// threads.
pub struct UnixSocketServer<App: Application> {
    app: App,
    listener: UnixListener,
    read_buf_size: usize,
}

impl<App: Application> UnixSocketServer<App> {
    pub(in crate::server) fn bind(
        app: App,
        socket_file: &Path,
        read_buf_size: usize,
    ) -> Result<UnixSocketServer<App>, Error> {
        let listener = UnixListener::bind(socket_file).map_err(Error::io)?;
        let socket_file = socket_file.to_path_buf();
        info!(
            "ABCI Unix server running at {:?}",
            socket_file.to_str().expect("wrong socket path")
        );

        let server = UnixSocketServer {
            app,
            listener,
            read_buf_size,
        };
        Ok(server)
    }
    /// Initiate a blocking listener for incoming connections.
    pub fn listen(self) -> Result<(), Error> {
        let listener = self.listener;
        for stream in listener.incoming() {
            info!("Incoming Unix connection");
            match stream {
                Ok(stream) => {
                    let app = self.app.clone();
                    let thread =
                        ClientThread::new(stream, String::from("unix"), app, self.read_buf_size);
                    thread::spawn(move || ClientThread::handle_client(thread));
                },
                Err(err) => error!("failed to process incoming connection: {}", err),
            }
        }

        Ok(())
    }
}

impl ReadWriter for UnixStream {
    fn clone(&self) -> Self {
        self.try_clone().expect("cannot clone UnixStream")
    }
}
