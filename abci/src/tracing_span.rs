use prost::Message;
use tenderdash_proto::abci::{self, request::Value};
use tracing::Level;

const SPAN_NAME: &str = "abci";
const LEVEL: Level = Level::ERROR;

macro_rules! block_span {
    ($request: expr, $endpoint:expr, $request_id:expr) => {
        tracing::span!(
            LEVEL,
            SPAN_NAME,
            endpoint = $endpoint,
            request_id = $request_id,
            height = $request.height,
            round = $request.round
        )
    };
}
/// Creates a new span for tracing.
///
/// This function creates a new `tracing::span::EnteredSpan` based on the
/// provided request. It uses the request to determine the endpoint and includes
/// a request ID in the span.
///
/// Request ID is deterministic and is based on the request value. It is
/// not guaranteed to be unique, as the same request can be sent multiple times.
/// However, it should be the same on all nodes for the same request.
///
/// The level of the span is set to ERROR, so it will be included on all log
/// levels.
///
/// # Arguments
///
/// * `request` - request to create a span for.
///
/// # Returns
///
/// An entered span which represents an active or entered span state.
///
/// # Examples
///
/// ```
/// # use tenderdash_proto::abci::{RequestInfo, Request, request::Value};
/// # use tenderdash_abci::tracing_span::span;
///
/// let request = Request {
///    value: Some(Value::Info(Default::default())),
/// };
/// let span = span(&request);
/// ```
pub fn span(request: &abci::Request) -> tracing::span::EnteredSpan
where
{
    let value = request.value.as_ref().expect("request value is missing");

    // we use md5 as we need 16-byte request id for uuid, and it doesn't have to be
    // cryptographically secure
    let mut md5 = lhash::Md5::new();
    md5.update(&request.encode_to_vec());
    let request_id = uuid::Uuid::from_bytes(md5.result())
        .as_hyphenated()
        .to_string();

    let endpoint = abci_method_name(value);

    let span = match value {
        Value::Info(_r) => tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id),
        Value::InitChain(_r) => {
            tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id)
        },
        Value::PrepareProposal(r) => block_span!(r, endpoint, request_id),
        Value::ProcessProposal(r) => block_span!(r, endpoint, request_id),
        Value::ExtendVote(r) => block_span!(r, endpoint, request_id),
        Value::VerifyVoteExtension(r) => block_span!(r, endpoint, request_id),
        Value::FinalizeBlock(r) => block_span!(r, endpoint, request_id),
        Value::CheckTx(_r) => {
            tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id)
        },
        Value::Query(r) => {
            tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id, path = r.path)
        },
        _ => tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id),
    };

    span.entered()
}

fn abci_method_name(request: &Value) -> String {
    match request {
        Value::ApplySnapshotChunk(_) => "ApplySnapshotChunk",
        Value::CheckTx(_) => "CheckTx",
        Value::Echo(_) => "Echo",
        Value::ExtendVote(_) => "ExtendVote",
        Value::FinalizeBlock(_) => "FinalizeBlock",
        Value::Flush(_) => "Flush",
        Value::Info(_) => "Info",
        Value::InitChain(_) => "InitChain",
        Value::ListSnapshots(_) => "ListSnapshots",
        Value::LoadSnapshotChunk(_) => "LoadSnapshotChunk",
        Value::OfferSnapshot(_) => "OfferSnapshot",
        Value::PrepareProposal(_) => "PrepareProposal",
        Value::ProcessProposal(_) => "ProcessProposal",
        Value::Query(_) => "Query",
        Value::VerifyVoteExtension(_) => "VerifyVoteExtension",
    }
    .to_string()
}
