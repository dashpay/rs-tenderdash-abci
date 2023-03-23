//! ABCI application interface.
use std::panic::RefUnwindSafe;

use crate::{proto, Error};

/// An ABCI application.
pub trait Application {
    /// Echo back the same message as provided in the request.
    fn echo(&self, request: proto::abci::RequestEcho) -> proto::abci::ResponseEcho {
        proto::abci::ResponseEcho {
            message: request.message,
        }
    }

    /// Signals that messages queued on the client should be flushed to the
    /// server.
    fn flush(&self, _request: proto::abci::RequestFlush) -> proto::abci::ResponseFlush {
        proto::abci::ResponseFlush {}
    }

    /// Provide information about the ABCI application.
    fn info(&self, _request: proto::abci::RequestInfo) -> proto::abci::ResponseInfo {
        Default::default()
    }

    /// Called once upon genesis.
    fn init_chain(
        &self,
        _request: proto::abci::RequestInitChain,
    ) -> proto::abci::ResponseInitChain {
        Default::default()
    }

    /// Query the application for data at the current or past height.
    fn query(&self, _request: proto::abci::RequestQuery) -> proto::abci::ResponseQuery {
        Default::default()
    }

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: proto::abci::RequestCheckTx) -> proto::abci::ResponseCheckTx {
        Default::default()
    }

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(
        &self,
        _request: proto::abci::RequestListSnapshots,
    ) -> proto::abci::ResponseListSnapshots {
        Default::default()
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(
        &self,
        _request: proto::abci::RequestOfferSnapshot,
    ) -> proto::abci::ResponseOfferSnapshot {
        Default::default()
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(
        &self,
        _request: proto::abci::RequestLoadSnapshotChunk,
    ) -> proto::abci::ResponseLoadSnapshotChunk {
        Default::default()
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: proto::abci::RequestApplySnapshotChunk,
    ) -> proto::abci::ResponseApplySnapshotChunk {
        Default::default()
    }

    fn extend_vote(
        &self,
        _request: proto::abci::RequestExtendVote,
    ) -> proto::abci::ResponseExtendVote {
        Default::default()
    }

    fn finalize_block(
        &self,
        _request: proto::abci::RequestFinalizeBlock,
    ) -> proto::abci::ResponseFinalizeBlock {
        Default::default()
    }

    fn prepare_proposal(
        &self,
        _request: proto::abci::RequestPrepareProposal,
    ) -> proto::abci::ResponsePrepareProposal {
        Default::default()
    }

    fn process_proposal(
        &self,
        _request: proto::abci::RequestProcessProposal,
    ) -> proto::abci::ResponseProcessProposal {
        Default::default()
    }

    fn verify_vote_extension(
        &self,
        _request: proto::abci::RequestVerifyVoteExtension,
    ) -> proto::abci::ResponseVerifyVoteExtension {
        Default::default()
    }
}

pub trait RequestDispatcher: RefUnwindSafe {
    /// Executes the relevant application method based on the type of the
    /// request, and produces the corresponding response.
    ///
    /// `RequestDispatcher` can indicate that it will no longer process new
    /// requests by returning `None` variant.
    fn handle(&self, request: proto::abci::Request)
        -> Result<Option<proto::abci::Response>, Error>;
}

// Implement `RequestDispatcher` for all `Application`s.
impl<A: Application + RefUnwindSafe> RequestDispatcher for A {
    fn handle(
        &self,
        request: proto::abci::Request,
    ) -> Result<Option<proto::abci::Response>, Error> {
        tracing::trace!("Incoming request: {:?}", request);
        let value = match request.value.unwrap() {
            proto::abci::request::Value::Echo(req) => {
                proto::abci::response::Value::Echo(self.echo(req))
            },
            proto::abci::request::Value::Flush(req) => {
                proto::abci::response::Value::Flush(self.flush(req))
            },
            proto::abci::request::Value::Info(req) => {
                proto::abci::response::Value::Info(self.info(req))
            },
            proto::abci::request::Value::InitChain(req) => {
                proto::abci::response::Value::InitChain(self.init_chain(req))
            },
            proto::abci::request::Value::Query(req) => {
                proto::abci::response::Value::Query(self.query(req))
            },
            proto::abci::request::Value::CheckTx(req) => {
                proto::abci::response::Value::CheckTx(self.check_tx(req))
            },
            proto::abci::request::Value::OfferSnapshot(req) => {
                proto::abci::response::Value::OfferSnapshot(self.offer_snapshot(req))
            },
            proto::abci::request::Value::LoadSnapshotChunk(req) => {
                proto::abci::response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req))
            },
            proto::abci::request::Value::ApplySnapshotChunk(req) => {
                proto::abci::response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req))
            },
            proto::abci::request::Value::ListSnapshots(req) => {
                proto::abci::response::Value::ListSnapshots(self.list_snapshots(req))
            },
            proto::abci::request::Value::PrepareProposal(req) => {
                proto::abci::response::Value::PrepareProposal(self.prepare_proposal(req))
            },
            proto::abci::request::Value::ProcessProposal(req) => {
                proto::abci::response::Value::ProcessProposal(self.process_proposal(req))
            },
            proto::abci::request::Value::FinalizeBlock(req) => {
                proto::abci::response::Value::FinalizeBlock(self.finalize_block(req))
            },
            proto::abci::request::Value::ExtendVote(req) => {
                proto::abci::response::Value::ExtendVote(self.extend_vote(req))
            },
            proto::abci::request::Value::VerifyVoteExtension(req) => {
                proto::abci::response::Value::VerifyVoteExtension(self.verify_vote_extension(req))
            },
        };
        tracing::trace!("Sending response: {:?}", value);

        Ok(Some(proto::abci::Response { value: Some(value) }))
    }
}
