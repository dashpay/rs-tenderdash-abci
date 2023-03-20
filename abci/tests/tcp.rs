use std::sync::Arc;

use tenderdash_abci::{Error, RequestDispatcher};

mod common;

use tenderdash_abci::proto;

#[cfg(feature = "docker-tests")]
#[test]
/// Test server listening on ipv4 address.
///
/// See [tcp_server_test()].
fn test_ipv4_server() {
    // we assume the host uses default Docker network configuration, with the host
    // using 172.17.0.1
    let bind_address = format!("tcp://172.17.0.1:1234");

    tcp_server_test("v4", bind_address.as_str());
}

#[cfg(feature = "docker-tests")]
#[test]
/// Test server listening on ipv6 address.
///
/// See [tcp_server_test()].
fn test_ipv6_server() {
    // we assume the host uses default Docker network configuration, with the host
    // using 172.17.0.1. This is IPv6 notation of the IPv4 address.
    let bind_address = format!("tcp://[::ffff:ac11:1]:5678");

    tcp_server_test("v6", bind_address.as_str());
}

#[cfg(feature = "docker-tests")]
/// Feature: ABCI App TCO server
///
/// * Given that we have Tenderdash instance using TCP connection to communicate
///   with ABCI APP
/// * When we estabilish connection with Tenderdash
/// * Then Tenderdash sends Info request
fn tcp_server_test(test_name: &str, bind_address: &str) {
    use tenderdash_abci::start_server;
    use tracing_subscriber::filter::LevelFilter;

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .try_init()
        .ok();

    let app = TestDispatcher {};

    let server = start_server(&bind_address, app).expect("server failed");
    let socket_uri = bind_address.to_string();
    let container_name = format!("tenderdash_{}", test_name);

    let td = Arc::new(common::docker::TenderdashDocker::new(
        &container_name,
        None,
        &socket_uri,
    ));

    common::docker::setup_td_logs_panic(&td);

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
