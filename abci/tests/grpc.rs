//! Test gRPC server for ABCI protocol.
//!
//! This test verifies that the gRPC server generated with tonic as part of the
//! tenderdash-proto crate can successfully connect to Tenderdash instance.
//!
//! This test should be implemented in the tenderdash-proto crate; however, it
//! is implemented here to use already existing docker container testing
//! logic.
#![cfg(feature = "grpc-server")]

use std::sync::Arc;

use tenderdash_abci::{
    proto::abci::{
        abci_application_server::AbciApplication, RequestEcho, RequestInfo, ResponseInfo,
    },
    CancellationToken,
};
mod common;
use tenderdash_abci::proto;
use tonic::{async_trait, Response, Status};

#[cfg(feature = "docker-tests")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
/// Test server listening on ipv4 address.
///
/// See [tcp_server_test()].
async fn test_ipv4_server() {
    // we assume the host uses default Docker network configuration, with the host
    // using 172.17.0.1
    let bind_address = "172.17.0.1:1234".to_string();

    grpc_server_test("v4", bind_address.as_str()).await;
}

#[cfg(feature = "docker-tests")]
#[tokio::test]
/// Test server listening on ipv6 address.
///
/// See [tcp_server_test()].
async fn test_ipv6_server() {
    // we assume the host uses default Docker network configuration, with the host
    // using 172.17.0.1. This is IPv6 notation of the IPv4 address.
    let bind_address = "[::ffff:ac11:1]:5678".to_string();

    grpc_server_test("v6", bind_address.as_str()).await;
}

#[cfg(feature = "docker-tests")]
/// Feature: ABCI App TCO server
///
/// * Given that we have Tenderdash instance using TCP connection to communicate
///   with ABCI APP
/// * When we estabilish connection with Tenderdash
/// * Then Tenderdash sends Info request
async fn grpc_server_test(test_name: &str, bind_address: &str) {
    use core::panic;

    use proto::abci::abci_application_server::AbciApplicationServer;
    use tonic::transport::Server;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("debug"))
        .with_ansi(true)
        .try_init()
        .ok();

    let cancel = CancellationToken::new();
    let app = TestApp {
        cancel: cancel.clone(),
    };

    let addr = bind_address.parse().expect("address must be valid");
    let server_cancel = cancel.clone();
    let server_handle = tokio::spawn(async move {
        tracing::debug!("starting gRPC server");
        Server::builder()
            .add_service(AbciApplicationServer::new(app))
            .serve_with_shutdown(addr, server_cancel.cancelled())
            .await
            .expect("server failed");
        tracing::debug!("gRPC server stopped");
    });

    let socket_uri = format!("grpc://{}", bind_address);
    let container_name = format!("tenderdash_{}", test_name);

    let td = tokio::task::spawn_blocking(move || {
        tracing::debug!("starting Tenderdash in Docker container");
        let td = Arc::new(common::docker::TenderdashDocker::new(
            &container_name,
            Some("feat-ABCI-protocol-env-var"),
            &socket_uri,
        ));
        common::docker::setup_td_logs_panic(&td);
        tracing::debug!("started Tenderdash in Docker container");

        td
    })
    .await
    .expect("start tenderdash");

    // tracing::debug!(?result, "connection handled");
    // assert!(matches!(result, Ok(())));
    tokio::select! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
            panic!("Test timed out");
        }
        _ = cancel.cancelled() => {
            tracing::debug!("CancellationToken cancelled");
        }
        ret = server_handle => {
            ret.expect("gRPC server failed");
        }
    }

    tokio::task::spawn_blocking(move || drop(td))
        .await
        .expect("tenderdash cleanup");

    tracing::info!("Test finished successfully");
}

pub struct TestApp {
    // when test succeeds, we cancel this token to finish it
    cancel: CancellationToken,
}
#[async_trait]
impl AbciApplication for TestApp {
    async fn echo(
        &self,
        request: tonic::Request<RequestEcho>,
    ) -> Result<tonic::Response<proto::abci::ResponseEcho>, Status> {
        tracing::info!(?request, "Echo called");
        Ok(Response::new(proto::abci::ResponseEcho {
            message: request.into_inner().message,
        }))
    }
    async fn info(
        &self,
        _request: tonic::Request<RequestInfo>,
    ) -> std::result::Result<tonic::Response<ResponseInfo>, tonic::Status> {
        tracing::info!("Info called, test successful");
        let resp = ResponseInfo {
            ..Default::default()
        };
        self.cancel.cancel();
        Ok(Response::new(resp))
    }
}
