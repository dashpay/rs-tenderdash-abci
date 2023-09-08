use tenderdash_proto::abci::request::Value;
use tracing::Level;

pub fn span<T>(request: T) -> tracing::span::EnteredSpan
where
    T: Into<Value>,
{
    let value = request.into();
    const SPAN_NAME: &str = "abci";
    const LEVEL: Level = Level::ERROR;
    let endpoint = abci_method_name(&value);
    let request_id = uuid::Uuid::new_v4().to_string();

    let span = match value {
        Value::Info(_r) => tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id),
        Value::InitChain(_r) => {
            tracing::span!(LEVEL, SPAN_NAME, endpoint, request_id)
        },
        Value::PrepareProposal(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                height = r.height,
                round = r.round,
            )
        },
        Value::ProcessProposal(r) => tracing::span!(
            LEVEL,
            SPAN_NAME,
            endpoint,
            request_id,
            height = r.height,
            round = r.round,
        ),
        Value::ExtendVote(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                height = r.height,
                round = r.round
            )
        },
        Value::VerifyVoteExtension(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                height = r.height,
                round = r.round
            )
        },
        Value::FinalizeBlock(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                height = r.height,
                round = r.round
            )
        },
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
