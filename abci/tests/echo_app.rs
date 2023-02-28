//! Integration tests for ABCI client/server.

#[cfg(all(feature = "client", feature = "echo-app"))]
mod echo_app_integration {
    use tenderdash_abci::{ClientBuilder, EchoApp, ServerBuilder};
    use tenderdash_proto::abci::RequestEcho;

    #[test]
    fn echo() {
        let server = ServerBuilder::default()
            .bind("127.0.0.1:0", EchoApp::default())
            .unwrap();
        let server_addr = server.local_addr();
        let _ = std::thread::spawn(move || server.listen());
        let mut client = ClientBuilder::default().connect(server_addr).unwrap();

        let response = client
            .echo(RequestEcho {
                message: "Hello ABCI!".to_string(),
            })
            .unwrap();
        assert_eq!(response.message, "Hello ABCI!");
    }
}
