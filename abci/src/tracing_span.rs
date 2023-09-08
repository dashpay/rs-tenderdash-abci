use hex::ToHex;
use tenderdash_proto::abci::request::Value;
use tracing::Level;
pub(super) fn span<T>(request: T) -> tracing::span::EnteredSpan
where
    T: Into<Value>,
{
    let value = request.into();
    const SPAN_NAME: &str = "abci";
    const LEVEL: Level = Level::ERROR;
    let endpoint = abci_method_name(&value);
    let request_id = uuid::Uuid::new_v4().to_string();

    let span = match value {
        Value::Info(r) => tracing::span!(
            LEVEL,
            SPAN_NAME,
            endpoint,
            request_id,
            tenderdash_version = r.version,
            block_version = r.block_version,
            p2p_version = r.p2p_version,
        ),
        Value::InitChain(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                chain_id = r.chain_id
            )
        },
        Value::PrepareProposal(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                height = r.height,
                round = r.round,
                quorum_hash = r.quorum_hash.encode_hex::<String>(),
                core_locked_height = r.core_chain_locked_height,
            )
        },
        Value::ProcessProposal(r) => tracing::span!(
            LEVEL,
            SPAN_NAME,
            endpoint,
            request_id,
            height = r.height,
            round = r.round,
            quorum_hash = r.quorum_hash.encode_hex::<String>(),
            core_locked_height = r.core_chain_locked_height,
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
        Value::CheckTx(r) => {
            tracing::span!(
                LEVEL,
                SPAN_NAME,
                endpoint,
                request_id,
                tx = r.tx.encode_hex::<String>()
            )
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
