#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(
        oneof = "request::Value",
        tags = "1, 2, 3, 4, 5, 7, 11, 12, 13, 14, 15, 16, 17, 18, 19"
    )]
    pub value: ::core::option::Option<request::Value>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Echo(super::RequestEcho),
        #[prost(message, tag = "2")]
        Flush(super::RequestFlush),
        #[prost(message, tag = "3")]
        Info(super::RequestInfo),
        #[prost(message, tag = "4")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag = "5")]
        Query(super::RequestQuery),
        #[prost(message, tag = "7")]
        CheckTx(super::RequestCheckTx),
        #[prost(message, tag = "11")]
        ListSnapshots(super::RequestListSnapshots),
        #[prost(message, tag = "12")]
        OfferSnapshot(super::RequestOfferSnapshot),
        #[prost(message, tag = "13")]
        LoadSnapshotChunk(super::RequestLoadSnapshotChunk),
        #[prost(message, tag = "14")]
        ApplySnapshotChunk(super::RequestApplySnapshotChunk),
        #[prost(message, tag = "15")]
        PrepareProposal(super::RequestPrepareProposal),
        #[prost(message, tag = "16")]
        ProcessProposal(super::RequestProcessProposal),
        #[prost(message, tag = "17")]
        ExtendVote(super::RequestExtendVote),
        #[prost(message, tag = "18")]
        VerifyVoteExtension(super::RequestVerifyVoteExtension),
        #[prost(message, tag = "19")]
        FinalizeBlock(super::RequestFinalizeBlock),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestEcho {
    /// A string to echo back
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFlush {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInfo {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub block_version: u64,
    #[prost(uint64, tag = "3")]
    pub p2p_version: u64,
    #[prost(string, tag = "4")]
    pub abci_version: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    /// Genesis time
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// ID of the blockchain.
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    /// Initial consensus-critical parameters.
    #[prost(message, optional, tag = "3")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
    /// Initial genesis validators, sorted by voting power.
    #[prost(message, optional, tag = "4")]
    pub validator_set: ::core::option::Option<ValidatorSetUpdate>,
    /// Serialized initial application state. JSON bytes.
    #[prost(bytes = "vec", tag = "5")]
    pub app_state_bytes: ::prost::alloc::vec::Vec<u8>,
    /// Height of the initial block (typically `1`).
    #[prost(int64, tag = "6")]
    pub initial_height: i64,
    /// Initial core chain lock height.
    #[prost(uint32, tag = "7")]
    pub initial_core_height: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestQuery {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bool, tag = "4")]
    pub prove: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCheckTx {
    #[prost(bytes = "vec", tag = "1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "CheckTxType", tag = "2")]
    pub r#type: i32,
}
/// lists available snapshots
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestListSnapshots {}
/// offers a snapshot to the application
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestOfferSnapshot {
    /// The snapshot offered for restoration.
    #[prost(message, optional, tag = "1")]
    pub snapshot: ::core::option::Option<Snapshot>,
    /// The light client-verified app hash for this height, from the blockchain.
    #[prost(bytes = "vec", tag = "2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
/// loads a snapshot chunk
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestLoadSnapshotChunk {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub chunk: u32,
}
/// Applies a snapshot chunk
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestApplySnapshotChunk {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestPrepareProposal {
    /// Currently configured maximum size in bytes taken by the modified transactions.
    /// The modified transactions cannot exceed this size.
    #[prost(int64, tag = "1")]
    pub max_tx_bytes: i64,
    /// Preliminary list of transactions that have been picked as part of the block to propose.
    /// Sent to the app for possible modifications.
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// Info about the last commit, obtained locally from Tendermint's data structures.
    #[prost(message, optional, tag = "3")]
    pub local_last_commit: ::core::option::Option<CommitInfo>,
    /// List of information about validators that acted incorrectly.
    #[prost(message, repeated, tag = "4")]
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
    /// The height of the block that will be proposed.
    #[prost(int64, tag = "5")]
    pub height: i64,
    /// Timestamp of the block that that will be proposed.
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// Merkle root of the next validator set.
    #[prost(bytes = "vec", tag = "7")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// Round number for the block.
    #[prost(int32, tag = "8")]
    pub round: i32,
    /// Core chain lock height to be used when signing this block.
    #[prost(uint32, tag = "9")]
    pub core_chain_locked_height: u32,
    /// ProTxHash of the original proposer of the block.
    #[prost(bytes = "vec", tag = "10")]
    pub proposer_pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    /// Proposer's latest available app protocol version.
    #[prost(uint64, tag = "11")]
    pub proposed_app_version: u64,
    /// App and block version used to generate the block.
    #[prost(message, optional, tag = "12")]
    pub version: ::core::option::Option<super::version::Consensus>,
    /// quorum_hash contains hash of validator quorum that will sign the block
    #[prost(bytes = "vec", tag = "13")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestProcessProposal {
    /// List of transactions that have been picked as part of the proposed
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// Info about the last commit, obtained from the information in the proposed block.
    #[prost(message, optional, tag = "2")]
    pub proposed_last_commit: ::core::option::Option<CommitInfo>,
    /// List of information about validators that acted incorrectly.
    #[prost(message, repeated, tag = "3")]
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
    /// The block header's hash of the proposed block.
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// The height of the proposed block.
    #[prost(int64, tag = "5")]
    pub height: i64,
    /// Round number for the block
    #[prost(int32, tag = "6")]
    pub round: i32,
    /// Timestamp included in the proposed block.
    #[prost(message, optional, tag = "7")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// Merkle root of the next validator set.
    #[prost(bytes = "vec", tag = "8")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// Core chain lock height to be used when signing this block.
    #[prost(uint32, tag = "9")]
    pub core_chain_locked_height: u32,
    /// Next core-chain-lock-update for validation in ABCI.
    #[prost(message, optional, tag = "10")]
    pub core_chain_lock_update: ::core::option::Option<super::types::CoreChainLock>,
    /// ProTxHash of the original proposer of the block.
    #[prost(bytes = "vec", tag = "11")]
    pub proposer_pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    /// Proposer's latest available app protocol version.
    #[prost(uint64, tag = "12")]
    pub proposed_app_version: u64,
    /// App and block version used to generate the block.
    #[prost(message, optional, tag = "13")]
    pub version: ::core::option::Option<super::version::Consensus>,
    /// quorum_hash contains hash of validator quorum that will sign the block
    #[prost(bytes = "vec", tag = "14")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
}
/// Extends a vote with application-side injection
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestExtendVote {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "2")]
    pub height: i64,
    /// Round number for the block
    #[prost(int32, tag = "3")]
    pub round: i32,
}
/// Verify the vote extension
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestVerifyVoteExtension {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub validator_pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// Round number for the block
    #[prost(int32, tag = "4")]
    pub round: i32,
    #[prost(message, repeated, tag = "5")]
    pub vote_extensions: ::prost::alloc::vec::Vec<ExtendVoteExtension>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFinalizeBlock {
    /// Info about the current commit
    #[prost(message, optional, tag = "1")]
    pub commit: ::core::option::Option<CommitInfo>,
    /// List of information about validators that acted incorrectly.
    #[prost(message, repeated, tag = "2")]
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
    /// The block header's hash. Present for convenience (can be derived from the block header).
    #[prost(bytes = "vec", tag = "3")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// The height of the finalized block.
    #[prost(int64, tag = "4")]
    pub height: i64,
    /// Round number for the block
    #[prost(int32, tag = "5")]
    pub round: i32,
    /// The block that was finalized
    #[prost(message, optional, tag = "6")]
    pub block: ::core::option::Option<super::types::Block>,
    /// The block ID that was finalized
    #[prost(message, optional, tag = "7")]
    pub block_id: ::core::option::Option<super::types::BlockId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(
        oneof = "response::Value",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16"
    )]
    pub value: ::core::option::Option<response::Value>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Exception(super::ResponseException),
        #[prost(message, tag = "2")]
        Echo(super::ResponseEcho),
        #[prost(message, tag = "3")]
        Flush(super::ResponseFlush),
        #[prost(message, tag = "4")]
        Info(super::ResponseInfo),
        #[prost(message, tag = "5")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag = "6")]
        Query(super::ResponseQuery),
        #[prost(message, tag = "7")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag = "8")]
        ListSnapshots(super::ResponseListSnapshots),
        #[prost(message, tag = "9")]
        OfferSnapshot(super::ResponseOfferSnapshot),
        #[prost(message, tag = "10")]
        LoadSnapshotChunk(super::ResponseLoadSnapshotChunk),
        #[prost(message, tag = "11")]
        ApplySnapshotChunk(super::ResponseApplySnapshotChunk),
        #[prost(message, tag = "12")]
        PrepareProposal(super::ResponsePrepareProposal),
        #[prost(message, tag = "13")]
        ProcessProposal(super::ResponseProcessProposal),
        #[prost(message, tag = "14")]
        ExtendVote(super::ResponseExtendVote),
        #[prost(message, tag = "15")]
        VerifyVoteExtension(super::ResponseVerifyVoteExtension),
        #[prost(message, tag = "16")]
        FinalizeBlock(super::ResponseFinalizeBlock),
    }
}
/// nondeterministic
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseException {
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEcho {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFlush {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInfo {
    #[prost(string, tag = "1")]
    pub data: ::prost::alloc::string::String,
    /// this is the software version of the application.
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub app_version: u64,
    #[prost(int64, tag = "4")]
    pub last_block_height: i64,
    #[prost(bytes = "vec", tag = "5")]
    pub last_block_app_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    /// Initial consensus-critical parameters (optional).
    #[prost(message, optional, tag = "1")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
    /// Initial application hash.
    #[prost(bytes = "vec", tag = "2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// Initial validator set (optional).
    #[prost(message, optional, tag = "3")]
    pub validator_set_update: ::core::option::Option<ValidatorSetUpdate>,
    /// Initial core chain lock update.
    #[prost(message, optional, tag = "4")]
    pub next_core_chain_lock_update: ::core::option::Option<super::types::CoreChainLock>,
    /// Initial height of core lock.
    #[prost(uint32, tag = "5")]
    pub initial_core_height: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseQuery {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// nondeterministic
    #[prost(string, tag = "2")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "4")]
    pub index: i64,
    #[prost(bytes = "vec", tag = "5")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "7")]
    pub proof_ops: ::core::option::Option<super::crypto::ProofOps>,
    #[prost(int64, tag = "8")]
    pub height: i64,
    #[prost(string, tag = "9")]
    pub codespace: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "4")]
    pub gas_wanted: i64,
    #[prost(string, tag = "5")]
    pub codespace: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub sender: ::prost::alloc::string::String,
    #[prost(int64, tag = "7")]
    pub priority: i64,
    /// mempool_error is set by Tendermint. ABCI applications creating a ResponseCheckTX should not set mempool_error.
    #[prost(string, tag = "8")]
    pub mempool_error: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseListSnapshots {
    #[prost(message, repeated, tag = "1")]
    pub snapshots: ::prost::alloc::vec::Vec<Snapshot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseOfferSnapshot {
    #[prost(enumeration = "response_offer_snapshot::Result", tag = "1")]
    pub result: i32,
}
/// Nested message and enum types in `ResponseOfferSnapshot`.
pub mod response_offer_snapshot {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Result {
        /// Unknown result, abort all snapshot restoration
        Unknown = 0,
        /// Snapshot accepted, apply chunks
        Accept = 1,
        /// Abort all snapshot restoration
        Abort = 2,
        /// Reject this specific snapshot, try others
        Reject = 3,
        /// Reject all snapshots of this format, try others
        RejectFormat = 4,
        /// Reject all snapshots from the sender(s), try others
        RejectSender = 5,
    }
    impl Result {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Result::Unknown => "UNKNOWN",
                Result::Accept => "ACCEPT",
                Result::Abort => "ABORT",
                Result::Reject => "REJECT",
                Result::RejectFormat => "REJECT_FORMAT",
                Result::RejectSender => "REJECT_SENDER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "ABORT" => Some(Self::Abort),
                "REJECT" => Some(Self::Reject),
                "REJECT_FORMAT" => Some(Self::RejectFormat),
                "REJECT_SENDER" => Some(Self::RejectSender),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseLoadSnapshotChunk {
    #[prost(bytes = "vec", tag = "1")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseApplySnapshotChunk {
    #[prost(enumeration = "response_apply_snapshot_chunk::Result", tag = "1")]
    pub result: i32,
    /// Chunks to refetch and reapply
    #[prost(uint32, repeated, tag = "2")]
    pub refetch_chunks: ::prost::alloc::vec::Vec<u32>,
    /// Chunk senders to reject and ban
    #[prost(string, repeated, tag = "3")]
    pub reject_senders: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `ResponseApplySnapshotChunk`.
pub mod response_apply_snapshot_chunk {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Result {
        /// Unknown result, abort all snapshot restoration
        Unknown = 0,
        /// Chunk successfully accepted
        Accept = 1,
        /// Abort all snapshot restoration
        Abort = 2,
        /// Retry chunk (combine with refetch and reject)
        Retry = 3,
        /// Retry snapshot (combine with refetch and reject)
        RetrySnapshot = 4,
        /// Reject this snapshot, try others
        RejectSnapshot = 5,
    }
    impl Result {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Result::Unknown => "UNKNOWN",
                Result::Accept => "ACCEPT",
                Result::Abort => "ABORT",
                Result::Retry => "RETRY",
                Result::RetrySnapshot => "RETRY_SNAPSHOT",
                Result::RejectSnapshot => "REJECT_SNAPSHOT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "ABORT" => Some(Self::Abort),
                "RETRY" => Some(Self::Retry),
                "RETRY_SNAPSHOT" => Some(Self::RetrySnapshot),
                "REJECT_SNAPSHOT" => Some(Self::RejectSnapshot),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponsePrepareProposal {
    /// Possibly modified list of transactions that have been picked as part of the proposed block.
    #[prost(message, repeated, tag = "1")]
    pub tx_records: ::prost::alloc::vec::Vec<TxRecord>,
    /// The Merkle root hash of the application state.
    #[prost(bytes = "vec", tag = "2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// List of structures containing the data resulting from executing the transactions.
    #[prost(message, repeated, tag = "3")]
    pub tx_results: ::prost::alloc::vec::Vec<ExecTxResult>,
    /// Changes to consensus-critical gas, size, and other parameters that will be applied at next height.
    #[prost(message, optional, tag = "4")]
    pub consensus_param_updates: ::core::option::Option<super::types::ConsensusParams>,
    /// Core chain lock that will be used for generated block.
    #[prost(message, optional, tag = "5")]
    pub core_chain_lock_update: ::core::option::Option<super::types::CoreChainLock>,
    /// Changes to validator set that will be applied at next height.
    #[prost(message, optional, tag = "6")]
    pub validator_set_update: ::core::option::Option<ValidatorSetUpdate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseProcessProposal {
    /// `enum` that signals if the application finds the proposal valid.
    #[prost(enumeration = "response_process_proposal::ProposalStatus", tag = "1")]
    pub status: i32,
    /// The Merkle root hash of the application state.
    #[prost(bytes = "vec", tag = "2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// List of structures containing the data resulting from executing the transactions.
    #[prost(message, repeated, tag = "3")]
    pub tx_results: ::prost::alloc::vec::Vec<ExecTxResult>,
    /// Changes to consensus-critical gas, size, and other parameters.
    #[prost(message, optional, tag = "4")]
    pub consensus_param_updates: ::core::option::Option<super::types::ConsensusParams>,
    /// Changes to validator set (set voting power to 0 to remove).
    #[prost(message, optional, tag = "5")]
    pub validator_set_update: ::core::option::Option<ValidatorSetUpdate>,
}
/// Nested message and enum types in `ResponseProcessProposal`.
pub mod response_process_proposal {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum ProposalStatus {
        /// Unspecified error occurred
        Unknown = 0,
        /// Proposal accepted
        Accept = 1,
        /// Proposal is not valid; prevoting `nil`
        Reject = 2,
    }
    impl ProposalStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ProposalStatus::Unknown => "UNKNOWN",
                ProposalStatus::Accept => "ACCEPT",
                ProposalStatus::Reject => "REJECT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "REJECT" => Some(Self::Reject),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendVoteExtension {
    #[prost(enumeration = "super::types::VoteExtensionType", tag = "1")]
    pub r#type: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub extension: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseExtendVote {
    #[prost(message, repeated, tag = "1")]
    pub vote_extensions: ::prost::alloc::vec::Vec<ExtendVoteExtension>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseVerifyVoteExtension {
    #[prost(enumeration = "response_verify_vote_extension::VerifyStatus", tag = "1")]
    pub status: i32,
}
/// Nested message and enum types in `ResponseVerifyVoteExtension`.
pub mod response_verify_vote_extension {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum VerifyStatus {
        Unknown = 0,
        Accept = 1,
        Reject = 2,
    }
    impl VerifyStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                VerifyStatus::Unknown => "UNKNOWN",
                VerifyStatus::Accept => "ACCEPT",
                VerifyStatus::Reject => "REJECT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "REJECT" => Some(Self::Reject),
                _ => None,
            }
        }
    }
}
/// In same-block execution mode, Tendermint will log an error and ignore values for ResponseFinalizeBlock.app_hash,
/// ResponseFinalizeBlock.tx_results, ResponseFinalizeBlock.validator_updates, and ResponsePrepareProposal.consensus_param_updates,
/// as those must have been provided by PrepareProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFinalizeBlock {
    /// Type & Key-Value events for indexing
    #[prost(message, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    /// Blocks below this height may be removed. Defaults to `0` (retain all).
    #[prost(int64, tag = "2")]
    pub retain_height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitInfo {
    #[prost(int32, tag = "1")]
    pub round: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub block_signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub threshold_vote_extensions: ::prost::alloc::vec::Vec<super::types::VoteExtension>,
}
/// Event allows application developers to attach additional information to
/// ResponseCheckTx, ResponsePrepareProposal, ResponseProcessProposal
/// and ResponseFinalizeBlock.
///
/// Later, transactions may be queried using these events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: ::prost::alloc::vec::Vec<EventAttribute>,
}
/// EventAttribute is a single key-value pair, associated with an event.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(bool, tag = "3")]
    pub index: bool,
}
/// ExecTxResult contains results of executing one individual transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecTxResult {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    /// nondeterministic
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
}
/// TxResult contains results of executing the transaction.
///
/// One usage is indexing transaction results.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxResult {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(uint32, tag = "2")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub result: ::core::option::Option<ExecTxResult>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxRecord {
    #[prost(enumeration = "tx_record::TxAction", tag = "1")]
    pub action: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `TxRecord`.
pub mod tx_record {
    /// TxAction contains App-provided information on what to do with a transaction that is part of a raw proposal
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum TxAction {
        /// Unknown action
        Unknown = 0,
        /// The Application did not modify this transaction.
        Unmodified = 1,
        /// The Application added this transaction.
        Added = 2,
        /// The Application wants this transaction removed from the proposal and the mempool.
        Removed = 3,
    }
    impl TxAction {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                TxAction::Unknown => "UNKNOWN",
                TxAction::Unmodified => "UNMODIFIED",
                TxAction::Added => "ADDED",
                TxAction::Removed => "REMOVED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "UNMODIFIED" => Some(Self::Unmodified),
                "ADDED" => Some(Self::Added),
                "REMOVED" => Some(Self::Removed),
                _ => None,
            }
        }
    }
}
/// Validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The voting power
    #[prost(int64, tag = "1")]
    pub power: i64,
    #[prost(bytes = "vec", tag = "2")]
    pub pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
}
/// ValidatorUpdate
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorUpdate {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag = "2")]
    pub power: i64,
    #[prost(bytes = "vec", tag = "3")]
    pub pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    /// node_address is an URI containing address of validator (`proto://node_id@ip_address:port`), for example:
    ///    `tcp://f2dbd9b0a1f541a7c44d34a58674d0262f5feca5@12.34.5.6:1234`
    #[prost(string, tag = "4")]
    pub node_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSetUpdate {
    #[prost(message, repeated, tag = "1")]
    pub validator_updates: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(message, optional, tag = "2")]
    pub threshold_public_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(bytes = "vec", tag = "3")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ThresholdPublicKeyUpdate {
    #[prost(message, optional, tag = "1")]
    pub threshold_public_key: ::core::option::Option<super::crypto::PublicKey>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuorumHashUpdate {
    #[prost(bytes = "vec", tag = "1")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
}
/// VoteInfo
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteInfo {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    #[prost(bool, tag = "2")]
    pub signed_last_block: bool,
}
/// ExtendedVoteInfo
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedVoteInfo {
    /// The validator that sent the vote.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    /// Indicates whether the validator signed the last block, allowing for rewards based on validator availability.
    #[prost(bool, tag = "2")]
    pub signed_last_block: bool,
    /// Non-deterministic extension provided by the sending validator's application.
    #[prost(bytes = "vec", tag = "3")]
    pub vote_extension: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehavior {
    #[prost(enumeration = "MisbehaviorType", tag = "1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag = "2")]
    pub validator: ::core::option::Option<Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    /// <https://github.com/tendermint/tendermint/issues/4581>
    #[prost(int64, tag = "5")]
    pub total_voting_power: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Snapshot {
    /// The height at which the snapshot was taken
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// The application-specific snapshot format
    #[prost(uint32, tag = "2")]
    pub format: u32,
    /// Number of chunks in the snapshot
    #[prost(uint32, tag = "3")]
    pub chunks: u32,
    /// Arbitrary snapshot hash, equal only if identical
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// Arbitrary application metadata
    #[prost(bytes = "vec", tag = "5")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CheckTxType {
    New = 0,
    Recheck = 1,
}
impl CheckTxType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CheckTxType::New => "NEW",
            CheckTxType::Recheck => "RECHECK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NEW" => Some(Self::New),
            "RECHECK" => Some(Self::Recheck),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MisbehaviorType {
    Unknown = 0,
    DuplicateVote = 1,
    LightClientAttack = 2,
}
impl MisbehaviorType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MisbehaviorType::Unknown => "UNKNOWN",
            MisbehaviorType::DuplicateVote => "DUPLICATE_VOTE",
            MisbehaviorType::LightClientAttack => "LIGHT_CLIENT_ATTACK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "DUPLICATE_VOTE" => Some(Self::DuplicateVote),
            "LIGHT_CLIENT_ATTACK" => Some(Self::LightClientAttack),
            _ => None,
        }
    }
}
