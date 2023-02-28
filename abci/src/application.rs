//! ABCI application interface.

#[cfg(feature = "echo-app")]
pub mod echo;
#[cfg(feature = "kvstore-app")]
pub mod kvstore;

use tenderdash_proto::abci::{
    request::Value,
    response,
    // Requests
    Request,
    RequestApplySnapshotChunk,
    RequestCheckTx,
    RequestEcho,
    RequestExtendVote,
    RequestFinalizeBlock,
    RequestFlush,
    RequestInfo,
    RequestInitChain,
    RequestListSnapshots,
    RequestLoadSnapshotChunk,
    RequestOfferSnapshot,
    RequestPrepareProposal,
    RequestProcessProposal,
    RequestQuery,
    RequestVerifyVoteExtension,
    // Responses
    Response,
    ResponseApplySnapshotChunk,
    ResponseCheckTx,
    ResponseEcho,
    ResponseExtendVote,
    ResponseFinalizeBlock,
    ResponseFlush,
    ResponseInfo,
    ResponseInitChain,
    ResponseListSnapshots,
    ResponseLoadSnapshotChunk,
    ResponseOfferSnapshot,
    ResponsePrepareProposal,
    ResponseProcessProposal,
    ResponseQuery,
    ResponseVerifyVoteExtension,
};

/*
type Application interface {
    // Info/Query Connection
    Info(context.Context, *RequestInfo) (*ResponseInfo, error)    // Return application info
    Query(context.Context, *RequestQuery) (*ResponseQuery, error) // Query for state

    // Mempool Connection
    CheckTx(context.Context, *RequestCheckTx) (*ResponseCheckTx, error) // Validate a tx for the mempool

    // Consensus Connection
    InitChain(context.Context, *RequestInitChain) (*ResponseInitChain, error) // Initialize blockchain w validators/other info from TendermintCore
    PrepareProposal(context.Context, *RequestPrepareProposal) (*ResponsePrepareProposal, error)
    ProcessProposal(context.Context, *RequestProcessProposal) (*ResponseProcessProposal, error)
    // Create application specific vote extension
    ExtendVote(context.Context, *RequestExtendVote) (*ResponseExtendVote, error)
    // Verify application's vote extension data
    VerifyVoteExtension(context.Context, *RequestVerifyVoteExtension) (*ResponseVerifyVoteExtension, error)
    // Deliver the decided block with its txs to the Application
    FinalizeBlock(context.Context, *RequestFinalizeBlock) (*ResponseFinalizeBlock, error)

    // State Sync Connection
    ListSnapshots(context.Context, *RequestListSnapshots) (*ResponseListSnapshots, error)                // List available snapshots
    OfferSnapshot(context.Context, *RequestOfferSnapshot) (*ResponseOfferSnapshot, error)                // Offer a snapshot to the application
    LoadSnapshotChunk(context.Context, *RequestLoadSnapshotChunk) (*ResponseLoadSnapshotChunk, error)    // Load a snapshot chunk
    ApplySnapshotChunk(context.Context, *RequestApplySnapshotChunk) (*ResponseApplySnapshotChunk, error) // Apply a shapshot chunk
}
*/

/// An ABCI application.
///
/// Applications are `Send` + `Clone` + `'static` because they are cloned for
/// each incoming connection to the ABCI [`Server`]. It is up to the
/// application developer to manage shared state between these clones of their
/// application.
///
/// [`Server`]: crate::Server
pub trait Application: Send + Clone + 'static {
    /// Echo back the same message as provided in the request.
    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    /// Signals that messages queued on the client should be flushed to the server.
    fn flush(&self, _request: RequestFlush) -> ResponseFlush {
        ResponseFlush {}
    }

    /// Provide information about the ABCI application.
    fn info(&self, _request: RequestInfo) -> ResponseInfo {
        Default::default()
    }

    /// Called once upon genesis.
    fn init_chain(&self, _request: RequestInitChain) -> ResponseInitChain {
        Default::default()
    }

    /// Query the application for data at the current or past height.
    fn query(&self, _request: RequestQuery) -> ResponseQuery {
        Default::default()
    }

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
        Default::default()
    }

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(&self, _request: RequestListSnapshots) -> ResponseListSnapshots {
        Default::default()
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(&self, _request: RequestOfferSnapshot) -> ResponseOfferSnapshot {
        Default::default()
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(&self, _request: RequestLoadSnapshotChunk) -> ResponseLoadSnapshotChunk {
        Default::default()
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        Default::default()
    }

    fn extend_vote(&self, _request: RequestExtendVote) -> ResponseExtendVote {
        Default::default()
    }

    fn finalize_block(&self, _request: RequestFinalizeBlock) -> ResponseFinalizeBlock {
        Default::default()
    }

    fn prepare_proposal(&self, _request: RequestPrepareProposal) -> ResponsePrepareProposal {
        Default::default()
    }

    fn process_proposal(&self, _request: RequestProcessProposal) -> ResponseProcessProposal {
        Default::default()
    }

    fn verify_vote_extension(
        &self,
        _request: RequestVerifyVoteExtension,
    ) -> ResponseVerifyVoteExtension {
        Default::default()
    }
}

/// Provides a mechanism for the [`Server`] to execute incoming requests while
/// expecting the correct response types.
pub trait RequestDispatcher {
    /// Executes the relevant application method based on the type of the
    /// request, and produces the corresponding response.
    fn handle(&self, request: Request) -> Response;
}

// Implement `RequestDispatcher` for all `Application`s.
impl<A: Application> RequestDispatcher for A {
    fn handle(&self, request: Request) -> Response {
        tracing::debug!("Incoming request: {:?}", request);
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req)),
                Value::Flush(req) => response::Value::Flush(self.flush(req)),
                Value::Info(req) => response::Value::Info(self.info(req)),
                Value::InitChain(req) => response::Value::InitChain(self.init_chain(req)),
                Value::Query(req) => response::Value::Query(self.query(req)),
                Value::CheckTx(req) => response::Value::CheckTx(self.check_tx(req)),
                Value::OfferSnapshot(req) => {
                    response::Value::OfferSnapshot(self.offer_snapshot(req))
                },
                Value::LoadSnapshotChunk(req) => {
                    response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req))
                },
                Value::ApplySnapshotChunk(req) => {
                    response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req))
                },
                Value::ListSnapshots(req) => {
                    response::Value::ListSnapshots(self.list_snapshots(req))
                },
                Value::PrepareProposal(req) => {
                    response::Value::PrepareProposal(self.prepare_proposal(req))
                },
                Value::ProcessProposal(req) => {
                    response::Value::ProcessProposal(self.process_proposal(req))
                },
                Value::FinalizeBlock(req) => {
                    response::Value::FinalizeBlock(self.finalize_block(req))
                },
                Value::ExtendVote(req) => response::Value::ExtendVote(self.extend_vote(req)),
                Value::VerifyVoteExtension(req) => {
                    response::Value::VerifyVoteExtension(self.verify_vote_extension(req))
                },
            }),
        }
    }
}
