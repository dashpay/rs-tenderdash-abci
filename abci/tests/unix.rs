use std::{path::Path, sync::Arc};

use tenderdash_abci::{start_unix, Error, RequestDispatcher};
mod common;
use tenderdash_abci::proto;

const SOCKET: &str = "/tmp/abci.sock";

#[cfg(feature = "docker-tests")]
#[test]
/// Feature: ABCI App socket server
///
/// * Given that we have Tenderdash instance using Unix Sockets to communicate
///   with ABCI APP
/// * When we estabilish connection with Tenderdash
/// * Then Tenderdash sends Info request
fn test_unix_socket_server() {
    use std::{fs, os::unix::prelude::PermissionsExt};

    use tracing_subscriber::filter::LevelFilter;

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let socket = Path::new(SOCKET);
    let app = TestDispatcher {};
    let server = start_unix(socket, app).expect("server failed");

    let perms = fs::Permissions::from_mode(0o777);
    fs::set_permissions(socket, perms).expect("set perms");

    let socket_uri = format!("unix://{}", socket.to_str().unwrap());
    let td = Arc::new(common::docker::TenderdashDocker::new(
        "fix-docker-init",
        &socket_uri,
    ));

    common::docker::setup_td_logs_panic(&td);

    assert!(matches!(server.handle_connection(), Ok(())));
}

/// Returns error containing string [`INFO_CALLED_ERROR`] when Tenderdash calls
/// Info() endpoint. All other requests return
/// Error::malformed_server_response()
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
