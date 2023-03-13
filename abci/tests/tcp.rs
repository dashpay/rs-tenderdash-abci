use tenderdash_abci::{error::Error, RequestDispatcher};
use tenderdash_proto::abci::request::Value;
use tracing_subscriber::filter::LevelFilter;

const INFO_CALLED_ERROR: &str = "info method called";

mod common;

#[cfg(feature = "docker-tests")]
#[test]
/// Feature: ABCI App TCO server
///
/// * Given that we have Tenderdash instance using TCP connection to communicate with ABCI APP
/// * When we estabilish connection with Tenderdash
/// * Then Tenderdash sends Info request
fn test_tcp_server() {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use tenderdash_abci::server::start_tcp;

    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let app = TestDispatcher {};
    // we assume the host uses default Docker network configuration, with the host using
    // 172.17.0.1
    let addr = SocketAddrV4::new(Ipv4Addr::new(172, 17, 0, 1), 1234);
    let server = start_tcp(addr, app).expect("server failed");
    let socket_uri = format!("tcp://{}", addr.to_string());
    let _td = common::docker::TenderdashDocker::new("fix-docker-init", &socket_uri);

    match server.handle_connection() {
        Ok(_) => (),
        Err(e) => {
            assert!(e.to_string().contains(INFO_CALLED_ERROR));
        },
    };
}

/// Returns error containing string [`INFO_CALLED_ERROR`] when Tenderdash calls Info()
/// endpoint. All other requests return Error::malformed_server_response()
pub struct TestDispatcher {}

impl RequestDispatcher for TestDispatcher {
    fn handle(
        &self,
        request: tenderdash_proto::abci::Request,
    ) -> Result<tenderdash_proto::abci::Response, tenderdash_abci::Error> {
        match request.value.unwrap() {
            Value::Info(_) => {
                return Err(Error::generic(String::from(INFO_CALLED_ERROR)));
            },
            _ => {
                return Err(Error::malformed_server_response());
            },
        };
    }
}
