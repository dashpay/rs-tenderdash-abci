//! ABCI application interface.
use tenderdash_proto::abci as proto;

use crate::Error;

/// An ABCI application.
pub trait Application {
    /// Echo back the same message as provided in the request.
    fn echo(&self, request: proto::RequestEcho) -> proto::ResponseEcho {
        proto::ResponseEcho {
            message: request.message,
        }
    }

    /// Signals that messages queued on the client should be flushed to the
    /// server.
    fn flush(&self, _request: proto::RequestFlush) -> proto::ResponseFlush {
        proto::ResponseFlush {}
    }

    /// Provide information about the ABCI application.
    fn info(&self, _request: proto::RequestInfo) -> proto::ResponseInfo {
        Default::default()
    }

    /// Called once upon genesis.
    fn init_chain(&self, _request: proto::RequestInitChain) -> proto::ResponseInitChain {
        Default::default()
    }

    /// Query the application for data at the current or past height.
    fn query(&self, _request: proto::RequestQuery) -> proto::ResponseQuery {
        Default::default()
    }

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: proto::RequestCheckTx) -> proto::ResponseCheckTx {
        Default::default()
    }

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(
        &self,
        _request: proto::RequestListSnapshots,
    ) -> proto::ResponseListSnapshots {
        Default::default()
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(
        &self,
        _request: proto::RequestOfferSnapshot,
    ) -> proto::ResponseOfferSnapshot {
        Default::default()
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(
        &self,
        _request: proto::RequestLoadSnapshotChunk,
    ) -> proto::ResponseLoadSnapshotChunk {
        Default::default()
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: proto::RequestApplySnapshotChunk,
    ) -> proto::ResponseApplySnapshotChunk {
        Default::default()
    }

    fn extend_vote(&self, _request: proto::RequestExtendVote) -> proto::ResponseExtendVote {
        Default::default()
    }

    fn finalize_block(
        &self,
        _request: proto::RequestFinalizeBlock,
    ) -> proto::ResponseFinalizeBlock {
        Default::default()
    }

    fn prepare_proposal(
        &self,
        _request: proto::RequestPrepareProposal,
    ) -> proto::ResponsePrepareProposal {
        Default::default()
    }

    fn process_proposal(
        &self,
        _request: proto::RequestProcessProposal,
    ) -> proto::ResponseProcessProposal {
        Default::default()
    }

    fn verify_vote_extension(
        &self,
        _request: proto::RequestVerifyVoteExtension,
    ) -> proto::ResponseVerifyVoteExtension {
        Default::default()
    }
}

/// Provides a mechanism for the [`Server`] to execute incoming requests while
/// expecting the correct response types.
///
/// [`Server`]: crate::server::Server
pub trait RequestDispatcher {
    /// Executes the relevant application method based on the type of the
    /// request, and produces the corresponding response.
    fn handle(&self, request: proto::Request) -> Result<proto::Response, Error>;
}

// Implement `RequestDispatcher` for all `Application`s.
impl<A: Application> RequestDispatcher for A {
    fn handle(&self, request: proto::Request) -> Result<proto::Response, Error> {
        tracing::debug!("Incoming request: {:?}", request);
        let value = match request.value.unwrap() {
            proto::request::Value::Echo(req) => proto::response::Value::Echo(self.echo(req)),
            proto::request::Value::Flush(req) => proto::response::Value::Flush(self.flush(req)),
            proto::request::Value::Info(req) => proto::response::Value::Info(self.info(req)),
            proto::request::Value::InitChain(req) => {
                proto::response::Value::InitChain(self.init_chain(req))
            },
            proto::request::Value::Query(req) => proto::response::Value::Query(self.query(req)),
            proto::request::Value::CheckTx(req) => {
                proto::response::Value::CheckTx(self.check_tx(req))
            },
            proto::request::Value::OfferSnapshot(req) => {
                proto::response::Value::OfferSnapshot(self.offer_snapshot(req))
            },
            proto::request::Value::LoadSnapshotChunk(req) => {
                proto::response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req))
            },
            proto::request::Value::ApplySnapshotChunk(req) => {
                proto::response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req))
            },
            proto::request::Value::ListSnapshots(req) => {
                proto::response::Value::ListSnapshots(self.list_snapshots(req))
            },
            proto::request::Value::PrepareProposal(req) => {
                proto::response::Value::PrepareProposal(self.prepare_proposal(req))
            },
            proto::request::Value::ProcessProposal(req) => {
                proto::response::Value::ProcessProposal(self.process_proposal(req))
            },
            proto::request::Value::FinalizeBlock(req) => {
                proto::response::Value::FinalizeBlock(self.finalize_block(req))
            },
            proto::request::Value::ExtendVote(req) => {
                proto::response::Value::ExtendVote(self.extend_vote(req))
            },
            proto::request::Value::VerifyVoteExtension(req) => {
                proto::response::Value::VerifyVoteExtension(self.verify_vote_extension(req))
            },
        };

        Ok(proto::Response { value: Some(value) })
    }
}
