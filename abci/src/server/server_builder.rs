/// The size of the read buffer for each incoming connection to the ABCI
/// server (1MB).
pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

/// Allows us to configure and construct an ABCI server.
pub struct ServerBuilder {
    read_buf_size: usize,
}

impl ServerBuilder {
    /// Builder constructor.
    ///
    /// Allows you to specify the read buffer size used when reading chunks of
    /// incoming data from the client. This needs to be tuned for your
    /// application.
    pub fn new(read_buf_size: usize) -> Self {
        Self { read_buf_size }
    }

    /// Constructor for an ABCI server.
    ///
    /// Binds the server to the given address. You must subsequently call the
    /// [`Server::listen`] method in order for incoming connections' requests
    /// to be routed to the specified ABCI application.
    pub fn bind<Addr, App>(self, addr: Addr, app: App) -> Result<Server<App>, Error>
    where
        Addr: ToSocketAddrs,
        App: Application,
    {
        let listener = TcpListener::bind(addr).map_err(Error::io)?;
        let local_addr = listener.local_addr().map_err(Error::io)?.to_string();
        info!("ABCI server running at {}", local_addr);
        Ok(Server {
            app,
            listener,
            local_addr,
            read_buf_size: self.read_buf_size,
        })
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self {
            read_buf_size: DEFAULT_SERVER_READ_BUF_SIZE,
        }
    }
}
