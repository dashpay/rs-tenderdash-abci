mod common;

use tenderdash_abci::{proto, Error, RequestDispatcher};

#[cfg(feature = "docker-tests")]
#[test]
/// Feature: ABCI App TCO server
///
/// * Given that we have Tenderdash instance using TCP connection to communicate
///   with ABCI APP
/// * When we estabilish connection with Tenderdash
/// * Then Tenderdash sends Info request
fn test_tcp_server() {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use tenderdash_abci::start_tcp;
    use tracing_subscriber::filter::LevelFilter;

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let app = TestDispatcher {};
    // we assume the host uses default Docker network configuration, with the host
    // using 172.17.0.1
    let addr = SocketAddrV4::new(Ipv4Addr::new(172, 17, 0, 1), 1234);
    let server = start_tcp(addr, app).expect("server failed");
    let socket_uri = format!("tcp://{}", addr.to_string());
    let _td = common::docker::TenderdashDocker::new("fix-docker-init", &socket_uri);

    assert!(matches!(server.handle_connection(), Ok(())));
}

pub struct TestDispatcher {}

impl RequestDispatcher for TestDispatcher {
    fn handle(
        &self,
        request: proto::abci::Request,
    ) -> Result<Option<proto::abci::Response>, Error> {
        // Assert that Info request will is received and close the connection
        assert!(matches!(
            request.value,
            Some(proto::abci::request::Value::Info(_))
        ));
        Ok(None)
    }
}
