//! ABCI application interface.
use std::panic::RefUnwindSafe;

use tracing::debug;

use crate::proto::{
    abci,
    abci::{request, response},
};

/// An ABCI application.
pub trait Application {
    /// Echo back the same message as provided in the request.
    fn echo(
        &self,
        request: abci::RequestEcho,
    ) -> Result<abci::ResponseEcho, abci::ResponseException> {
        Ok(abci::ResponseEcho {
            message: request.message,
        })
    }

    /// Signals that messages queued on the client should be flushed to the
    /// server.
    fn flush(
        &self,
        _request: abci::RequestFlush,
    ) -> Result<abci::ResponseFlush, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Provide information about the ABCI application.
    fn info(
        &self,
        request: abci::RequestInfo,
    ) -> Result<abci::ResponseInfo, abci::ResponseException> {
        if !check_version(&request.abci_version) {
            return Err(abci::ResponseException {
                error: format!(
                    "version mismatch: tenderdash {} vs our {}",
                    request.version,
                    crate::proto::ABCI_VERSION
                ),
            });
        }

        Ok(Default::default())
    }

    /// Called once upon genesis.
    fn init_chain(
        &self,
        _request: abci::RequestInitChain,
    ) -> Result<abci::ResponseInitChain, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Query the application for data at the current or past height.
    fn query(
        &self,
        _request: abci::RequestQuery,
    ) -> Result<abci::ResponseQuery, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(
        &self,
        _request: abci::RequestCheckTx,
    ) -> Result<abci::ResponseCheckTx, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(
        &self,
        _request: abci::RequestListSnapshots,
    ) -> Result<abci::ResponseListSnapshots, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(
        &self,
        _request: abci::RequestOfferSnapshot,
    ) -> Result<abci::ResponseOfferSnapshot, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(
        &self,
        _request: abci::RequestLoadSnapshotChunk,
    ) -> Result<abci::ResponseLoadSnapshotChunk, abci::ResponseException> {
        Ok(Default::default())
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: abci::RequestApplySnapshotChunk,
    ) -> Result<abci::ResponseApplySnapshotChunk, abci::ResponseException> {
        Ok(Default::default())
    }

    fn extend_vote(
        &self,
        _request: abci::RequestExtendVote,
    ) -> Result<abci::ResponseExtendVote, abci::ResponseException> {
        Ok(Default::default())
    }

    fn finalize_block(
        &self,
        _request: abci::RequestFinalizeBlock,
    ) -> Result<abci::ResponseFinalizeBlock, abci::ResponseException> {
        Ok(Default::default())
    }

    fn prepare_proposal(
        &self,
        _request: abci::RequestPrepareProposal,
    ) -> Result<abci::ResponsePrepareProposal, abci::ResponseException> {
        Ok(Default::default())
    }

    fn process_proposal(
        &self,
        _request: abci::RequestProcessProposal,
    ) -> Result<abci::ResponseProcessProposal, abci::ResponseException> {
        Ok(Default::default())
    }

    fn verify_vote_extension(
        &self,
        _request: abci::RequestVerifyVoteExtension,
    ) -> Result<abci::ResponseVerifyVoteExtension, abci::ResponseException> {
        Ok(Default::default())
    }
}

pub trait RequestDispatcher: RefUnwindSafe {
    /// Executes the relevant application method based on the type of the
    /// request, and produces the corresponding response.
    ///
    /// `RequestDispatcher` can indicate that it will no longer process new
    /// requests by returning `None` variant.
    fn handle(&self, request: abci::Request) -> Option<abci::Response>;
}

// Implement `RequestDispatcher` for all `Application`s.
impl<A: Application + RefUnwindSafe> RequestDispatcher for A {
    fn handle(&self, request: abci::Request) -> Option<abci::Response> {
        tracing::trace!(?request, "received request");

        let response: Result<response::Value, abci::ResponseException> = match request.value? {
            request::Value::Echo(req) => self.echo(req).map(|v| v.into()),
            request::Value::Flush(req) => self.flush(req).map(|v| v.into()),
            request::Value::Info(req) => self.info(req).map(|v| v.into()),
            request::Value::InitChain(req) => self.init_chain(req).map(|v| v.into()),
            request::Value::Query(req) => self.query(req).map(|v| v.into()),
            request::Value::CheckTx(req) => self.check_tx(req).map(|v| v.into()),
            request::Value::OfferSnapshot(req) => self.offer_snapshot(req).map(|v| v.into()),
            request::Value::LoadSnapshotChunk(req) => {
                self.load_snapshot_chunk(req).map(|v| v.into())
            },
            request::Value::ApplySnapshotChunk(req) => {
                self.apply_snapshot_chunk(req).map(|v| v.into())
            },
            request::Value::ListSnapshots(req) => self.list_snapshots(req).map(|v| v.into()),
            request::Value::PrepareProposal(req) => self.prepare_proposal(req).map(|v| v.into()),
            request::Value::ProcessProposal(req) => self.process_proposal(req).map(|v| v.into()),
            request::Value::FinalizeBlock(req) => self.finalize_block(req).map(|v| v.into()),
            request::Value::ExtendVote(req) => self.extend_vote(req).map(|v| v.into()),
            request::Value::VerifyVoteExtension(req) => {
                self.verify_vote_extension(req).map(|v| v.into())
            },
        };

        let response = match response {
            Ok(v) => v,
            Err(e) => response::Value::from(e),
        };

        tracing::trace!(?response, "sending response");

        Some(abci::Response {
            value: Some(response),
        })
    }
}

/// Check if ABCI version sent by Tenderdash matches version of linked protobuf
/// data objects.
///
/// You should use this function inside [Application::info()] handler, to ensure
/// that the protocol versions match. Match is determined based on Semantic
/// Versioning rules, as defined for '^' operator.
///
/// ## Examples
///
/// ```should_panic
/// use tenderdash_abci::{check_version, Application};
/// use tenderdash_abci::proto::abci::{RequestInfo, ResponseInfo, ResponseException};
///
/// # let request = RequestInfo{
/// #  abci_version: String::from("108.234.356"),
/// #  ..Default::default()
/// # };
/// struct AbciApp{}
///
/// impl tenderdash_abci::Application for AbciApp {
///   fn info(&self, request: RequestInfo) -> Result<ResponseInfo, ResponseException> {
///     if !check_version(&request.abci_version) {
///       panic!("abci version mismatch");
///     }
///     Ok(Default::default())
///   }
/// }
///
/// # let app = AbciApp{};
/// # app.info(request);
/// ```
pub fn check_version(tenderdash_version: &str) -> bool {
    match_versions(tenderdash_version, tenderdash_proto::ABCI_VERSION)
}

fn match_versions(tenderdash_abci_requirement: &str, our_abci_version: &str) -> bool {
    let our_version =
        semver::Version::parse(our_abci_version).expect("cannot parse protobuf library version");

    let require = String::from("^") + tenderdash_abci_requirement;
    let td_version =
        semver::VersionReq::parse(require.as_str()).expect("cannot parse tenderdash version");

    debug!("ABCI version: required: {}, our: {}", require, our_version);

    td_version.matches(&our_version)
}

#[cfg(test)]
mod tests {
    use super::match_versions;

    /// test_versions! {} (td_version, our_version, expected); }
    macro_rules! test_versions {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (td, our, expect) = $value;
                assert_eq!(match_versions(td, our),expect);
            }
        )*
        }
    }

    test_versions! {
        test_versions_td_newer: ("0.1.2-dev.1", "0.1.0", false),
        test_versions_equal: ("0.1.0","0.1.0",true),
        test_versions_td_older: ("0.1.0","0.1.2",true),
        test_versions_equal_dev: ("0.1.0-dev.1","0.1.0-dev.1",true),
        test_versions_our_newer_dev: ("0.1.0-dev.1", "0.1.0-dev.2",true),
        test_versions_our_dev:("0.1.0","0.1.0-dev.1",false),
    }
}
