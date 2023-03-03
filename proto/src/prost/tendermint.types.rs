/// CoreChainLock represents a core chain lock for synchronization between state data and core chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CoreChainLock {
    #[prost(uint32, tag = "1")]
    pub core_block_height: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub core_block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteExtension {
    #[prost(enumeration = "VoteExtensionType", tag = "1")]
    pub r#type: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub extension: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum VoteExtensionType {
    Default = 0,
    ThresholdRecover = 1,
}
impl VoteExtensionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            VoteExtensionType::Default => "DEFAULT",
            VoteExtensionType::ThresholdRecover => "THRESHOLD_RECOVER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DEFAULT" => Some(Self::Default),
            "THRESHOLD_RECOVER" => Some(Self::ThresholdRecover),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSet {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(message, optional, tag = "2")]
    pub proposer: ::core::option::Option<Validator>,
    #[prost(int64, tag = "3")]
    pub total_voting_power: i64,
    #[prost(message, optional, tag = "4")]
    pub threshold_public_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int32, tag = "5")]
    pub quorum_type: i32,
    #[prost(bytes = "vec", tag = "6")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "7")]
    pub has_public_keys: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag = "2")]
    pub voting_power: i64,
    #[prost(int64, tag = "3")]
    pub proposer_priority: i64,
    #[prost(bytes = "vec", tag = "4")]
    pub pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    /// address of the Validator, correct URI (RFC 3986)
    #[prost(string, tag = "5")]
    pub node_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleValidator {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag = "2")]
    pub voting_power: i64,
}
/// PartsetHeader
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Part {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::crypto::Proof>,
}
/// BlockID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub part_set_header: ::core::option::Option<PartSetHeader>,
    /// state_id is a hash of fields required to validate state in light client.
    /// See types/stateid.go for details.
    #[prost(bytes = "vec", tag = "3")]
    pub state_id: ::prost::alloc::vec::Vec<u8>,
}
/// StateID represents essential information required to verify state, document and transactions.
/// It is meant to be used by light clients (like mobile apps) to verify proofs.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateId {
    /// AppVersion used when generating the block, equals to Header.Version.App.
    #[prost(fixed64, tag = "1")]
    pub app_version: u64,
    /// Height of block containing this state ID.
    #[prost(fixed64, tag = "2")]
    pub height: u64,
    /// AppHash used in current block, equal to Header.AppHash. 32 bytes.
    #[prost(bytes = "vec", tag = "3")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// CoreChainLockedHeight for the block, equal to Header.CoreChainLockedHeight.
    #[prost(fixed32, tag = "4")]
    pub core_chain_locked_height: u32,
    /// Time of the block.
    #[prost(message, optional, tag = "5")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
}
/// Header defines the structure of a Tendermint block header.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// basic block info
    #[prost(message, optional, tag = "1")]
    pub version: ::core::option::Option<super::version::Consensus>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// prev block info
    #[prost(message, optional, tag = "5")]
    pub last_block_id: ::core::option::Option<BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[prost(bytes = "vec", tag = "6")]
    pub last_commit_hash: ::prost::alloc::vec::Vec<u8>,
    /// transactions
    #[prost(bytes = "vec", tag = "7")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[prost(bytes = "vec", tag = "8")]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// validators for the next block
    #[prost(bytes = "vec", tag = "9")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus params for current block
    #[prost(bytes = "vec", tag = "10")]
    pub consensus_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus params for next block
    #[prost(bytes = "vec", tag = "11")]
    pub next_consensus_hash: ::prost::alloc::vec::Vec<u8>,
    /// state after txs from the previous block
    #[prost(bytes = "vec", tag = "12")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// root hash of all results from the txs from current block
    #[prost(bytes = "vec", tag = "13")]
    pub results_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[prost(bytes = "vec", tag = "14")]
    pub evidence_hash: ::prost::alloc::vec::Vec<u8>,
    /// proposer's latest available app protocol version
    #[prost(uint64, tag = "15")]
    pub proposed_app_version: u64,
    /// original proposer of the block
    #[prost(bytes = "vec", tag = "16")]
    pub proposer_pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "17")]
    pub core_chain_locked_height: u32,
}
/// Data contains the set of transactions included in the block
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// Txs that will be applied by state @ block.Height+1.
    /// NOTE: not all txs here are valid.  We're just agreeing on the order first.
    /// This means that block.AppHash does not include these txs.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// Vote represents a prevote, precommit, or commit vote from validators for
/// consensus.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    /// zero if vote is nil.
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(bytes = "vec", tag = "5")]
    pub validator_pro_tx_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int32, tag = "6")]
    pub validator_index: i32,
    #[prost(bytes = "vec", tag = "7")]
    pub block_signature: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension provided by the application. Only valid for precommit
    /// messages.
    /// Vote extension signature by the validator if they participated in
    /// consensus for the associated block. Only valid for precommit messages.
    #[prost(message, repeated, tag = "8")]
    pub vote_extensions: ::prost::alloc::vec::Vec<VoteExtension>,
}
/// Commit contains the evidence that a block was committed by a set of
/// validators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commit {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(message, optional, tag = "3")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(bytes = "vec", tag = "4")]
    pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub threshold_block_signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "6")]
    pub threshold_vote_extensions: ::prost::alloc::vec::Vec<VoteExtension>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    #[prost(int32, tag = "4")]
    pub pol_round: i32,
    #[prost(message, optional, tag = "5")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag = "6")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(bytes = "vec", tag = "7")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "8")]
    pub core_chain_locked_height: u32,
    #[prost(message, optional, tag = "9")]
    pub core_chain_lock_update: ::core::option::Option<CoreChainLock>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedHeader {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub commit: ::core::option::Option<Commit>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightBlock {
    #[prost(message, optional, tag = "1")]
    pub signed_header: ::core::option::Option<SignedHeader>,
    #[prost(message, optional, tag = "2")]
    pub validator_set: ::core::option::Option<ValidatorSet>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMeta {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(int64, tag = "2")]
    pub block_size: i64,
    #[prost(message, optional, tag = "3")]
    pub header: ::core::option::Option<Header>,
    #[prost(int64, tag = "4")]
    pub num_txs: i64,
    #[prost(int32, tag = "5")]
    pub round: i32,
    #[prost(bool, tag = "6")]
    pub has_core_chain_lock: bool,
}
/// TxProof represents a Merkle proof of the presence of a transaction in the
/// Merkle tree.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxProof {
    #[prost(bytes = "vec", tag = "1")]
    pub root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::crypto::Proof>,
}
/// BlockIdFlag indicates which BlockID the signature is for
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BlockIdFlag {
    Unknown = 0,
    Absent = 1,
    Commit = 2,
    Nil = 3,
}
impl BlockIdFlag {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BlockIdFlag::Unknown => "BLOCK_ID_FLAG_UNKNOWN",
            BlockIdFlag::Absent => "BLOCK_ID_FLAG_ABSENT",
            BlockIdFlag::Commit => "BLOCK_ID_FLAG_COMMIT",
            BlockIdFlag::Nil => "BLOCK_ID_FLAG_NIL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BLOCK_ID_FLAG_UNKNOWN" => Some(Self::Unknown),
            "BLOCK_ID_FLAG_ABSENT" => Some(Self::Absent),
            "BLOCK_ID_FLAG_COMMIT" => Some(Self::Commit),
            "BLOCK_ID_FLAG_NIL" => Some(Self::Nil),
            _ => None,
        }
    }
}
/// SignedMsgType is a type of signed message in the consensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SignedMsgType {
    Unknown = 0,
    /// Votes
    Prevote = 1,
    Precommit = 2,
    Commit = 3,
    /// Proposals
    Proposal = 32,
}
impl SignedMsgType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SignedMsgType::Unknown => "SIGNED_MSG_TYPE_UNKNOWN",
            SignedMsgType::Prevote => "SIGNED_MSG_TYPE_PREVOTE",
            SignedMsgType::Precommit => "SIGNED_MSG_TYPE_PRECOMMIT",
            SignedMsgType::Commit => "SIGNED_MSG_TYPE_COMMIT",
            SignedMsgType::Proposal => "SIGNED_MSG_TYPE_PROPOSAL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIGNED_MSG_TYPE_UNKNOWN" => Some(Self::Unknown),
            "SIGNED_MSG_TYPE_PREVOTE" => Some(Self::Prevote),
            "SIGNED_MSG_TYPE_PRECOMMIT" => Some(Self::Precommit),
            "SIGNED_MSG_TYPE_COMMIT" => Some(Self::Commit),
            "SIGNED_MSG_TYPE_PROPOSAL" => Some(Self::Proposal),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    #[prost(oneof = "evidence::Sum", tags = "1")]
    pub sum: ::core::option::Option<evidence::Sum>,
}
/// Nested message and enum types in `Evidence`.
pub mod evidence {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        DuplicateVoteEvidence(super::DuplicateVoteEvidence),
    }
}
/// DuplicateVoteEvidence contains evidence of a validator signed two conflicting
/// votes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DuplicateVoteEvidence {
    #[prost(message, optional, tag = "1")]
    pub vote_a: ::core::option::Option<Vote>,
    #[prost(message, optional, tag = "2")]
    pub vote_b: ::core::option::Option<Vote>,
    #[prost(int64, tag = "3")]
    pub total_voting_power: i64,
    #[prost(int64, tag = "4")]
    pub validator_power: i64,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceList {
    #[prost(message, repeated, tag = "1")]
    pub evidence: ::prost::alloc::vec::Vec<Evidence>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<Data>,
    #[prost(message, optional, tag = "3")]
    pub evidence: ::core::option::Option<EvidenceList>,
    #[prost(message, optional, tag = "4")]
    pub last_commit: ::core::option::Option<Commit>,
    #[prost(message, optional, tag = "5")]
    pub core_chain_lock: ::core::option::Option<CoreChainLock>,
}
/// ConsensusParams contains consensus critical parameters that determine the
/// validity of blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag = "2")]
    pub evidence: ::core::option::Option<EvidenceParams>,
    #[prost(message, optional, tag = "3")]
    pub validator: ::core::option::Option<ValidatorParams>,
    #[prost(message, optional, tag = "4")]
    pub version: ::core::option::Option<VersionParams>,
    #[prost(message, optional, tag = "5")]
    pub synchrony: ::core::option::Option<SynchronyParams>,
    #[prost(message, optional, tag = "6")]
    pub timeout: ::core::option::Option<TimeoutParams>,
    #[prost(message, optional, tag = "7")]
    pub abci: ::core::option::Option<AbciParams>,
}
/// BlockParams contains limits on the block size.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Max block size, in bytes.
    /// Note: must be greater than 0
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Max gas per block.
    /// Note: must be greater or equal to -1
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}
/// EvidenceParams determine how we handle evidence of malfeasance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceParams {
    /// Max age of evidence, in blocks.
    ///
    /// The basic formula for calculating this is: MaxAgeDuration / {average block
    /// time}.
    #[prost(int64, tag = "1")]
    pub max_age_num_blocks: i64,
    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake
    /// attacks](<https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed>).
    #[prost(message, optional, tag = "2")]
    pub max_age_duration: ::core::option::Option<
        super::super::google::protobuf::Duration,
    >,
    /// This sets the maximum size of total evidence in bytes that can be committed
    /// in a single block. and should fall comfortably under the max block bytes.
    /// Default is 1048576 or 1MB
    #[prost(int64, tag = "3")]
    pub max_bytes: i64,
}
/// ValidatorParams restrict the public key types validators can use.
/// NOTE: uses ABCI pubkey naming, not Amino names.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorParams {
    #[prost(string, repeated, tag = "1")]
    pub pub_key_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// VersionParams contains the ABCI application version.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionParams {
    #[prost(uint64, tag = "1")]
    pub app_version: u64,
}
/// HashedParams is a subset of ConsensusParams.
///
/// It is hashed into the Header.ConsensusHash.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HashedParams {
    #[prost(int64, tag = "1")]
    pub block_max_bytes: i64,
    #[prost(int64, tag = "2")]
    pub block_max_gas: i64,
}
/// SynchronyParams configure the bounds under which a proposed block's timestamp is considered valid.
/// These parameters are part of the proposer-based timestamps algorithm. For more information,
/// see the specification of proposer-based timestamps:
/// <https://github.com/tendermint/tendermint/tree/master/spec/consensus/proposer-based-timestamp>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SynchronyParams {
    /// message_delay bounds how long a proposal message may take to reach all validators on a network
    /// and still be considered valid.
    #[prost(message, optional, tag = "1")]
    pub message_delay: ::core::option::Option<super::super::google::protobuf::Duration>,
    /// precision bounds how skewed a proposer's clock may be from any validator
    /// on the network while still producing valid proposals.
    #[prost(message, optional, tag = "2")]
    pub precision: ::core::option::Option<super::super::google::protobuf::Duration>,
}
/// TimeoutParams configure the timeouts for the steps of the Tendermint consensus algorithm.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeoutParams {
    /// These fields configure the timeouts for the propose step of the Tendermint
    /// consensus algorithm: propose is the initial timeout and propose_delta
    /// determines how much the timeout grows in subsequent rounds.
    /// For the first round, this propose timeout is used and for every subsequent
    /// round, the timeout grows by propose_delta.
    ///
    /// For example:
    /// With propose = 10ms, propose_delta = 5ms, the first round's propose phase
    /// timeout would be 10ms, the second round's would be 15ms, the third 20ms and so on.
    ///
    /// If a node waiting for a proposal message does not receive one matching its
    /// current height and round before this timeout, the node will issue a
    /// nil prevote for the round and advance to the next step.
    #[prost(message, optional, tag = "1")]
    pub propose: ::core::option::Option<super::super::google::protobuf::Duration>,
    #[prost(message, optional, tag = "2")]
    pub propose_delta: ::core::option::Option<super::super::google::protobuf::Duration>,
    /// vote along with vote_delta configure the timeout for both of the prevote and
    /// precommit steps of the Tendermint consensus algorithm.
    ///
    /// These parameters influence the vote step timeouts in the the same way that
    /// the propose and propose_delta parameters do to the proposal step.
    ///
    /// The vote timeout does not begin until a quorum of votes has been received. Once
    /// a quorum of votes has been seen and this timeout elapses, Tendermint will
    /// procced to the next step of the consensus algorithm. If Tendermint receives
    /// all of the remaining votes before the end of the timeout, it will proceed
    /// to the next step immediately.
    #[prost(message, optional, tag = "3")]
    pub vote: ::core::option::Option<super::super::google::protobuf::Duration>,
    #[prost(message, optional, tag = "4")]
    pub vote_delta: ::core::option::Option<super::super::google::protobuf::Duration>,
    /// commit configures how long Tendermint will wait after receiving a quorum of
    /// precommits before beginning consensus for the next height. This can be
    /// used to allow slow precommits to arrive for inclusion in the next height before progressing.
    #[prost(message, optional, tag = "5")]
    pub commit: ::core::option::Option<super::super::google::protobuf::Duration>,
    /// bypass_commit_timeout configures the node to proceed immediately to
    /// the next height once the node has received all precommits for a block, forgoing
    /// the remaining commit timeout.
    /// Setting bypass_commit_timeout false (the default) causes Tendermint to wait
    /// for the full commit timeout.
    #[prost(bool, tag = "6")]
    pub bypass_commit_timeout: bool,
}
/// ABCIParams configure functionality specific to the Application Blockchain Interface.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciParams {
    /// Indicates if CheckTx should be called on all the transactions
    /// remaining in the mempool after a block is executed.
    #[prost(bool, tag = "1")]
    pub recheck_tx: bool,
}
