use std::path::Path;

use tenderdash_abci::{error::Error, server::start_unix, RequestDispatcher};
use tenderdash_proto::abci::request::Value;
use tracing_subscriber::filter::LevelFilter;

const SOCKET: &str = "/tmp/abci.sock";
const INFO_CALLED_ERROR: &str = "info method called";

mod common;

#[cfg(feature = "docker-tests")]
#[test]
/// Feature: ABCI App socket server
///
/// * Given that we have Tenderdash instance using Unix Sockets to communicate with ABCI APP
/// * When we estabilish connection with Tenderdash
/// * Then Tenderdash sends Info request
fn test_unix_socket_server() {
    use std::{fs, os::unix::prelude::PermissionsExt};

    let log_level = LevelFilter::DEBUG;
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let socket = Path::new(SOCKET);
    let app = TestDispatcher {};
    let server = start_unix(socket, app).expect("server failed");

    let perms = fs::Permissions::from_mode(0o777);
    fs::set_permissions(socket, perms).expect("set perms");

    let socket_uri = format!("unix://{}", socket.to_str().unwrap());
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
