/// Tenderdash protobuf version

/// Tenderdash repository URL.
pub const TENDERDASH_REPO: &str = "https://github.com/dashpay/tenderdash";
// Commitish formats:
// Tag: v0.34.0-rc4
// Branch: master
// Commit ID (full length): d7d0ffea13c60c98b812d243ba5a2c375f341c15
// Set env variable TENDERDASH_COMMITISH to override the default.
pub(crate) const DEFAULT_TENDERDASH_COMMITISH: &str = "v0.10-dev";

/// Predefined custom attributes for message annotations
const PRIMITIVE_ENUM: &str = r#"#[derive(::num_derive::FromPrimitive, ::num_derive::ToPrimitive)]"#;
const SERIALIZED: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;
const TYPE_TAG: &str = r#"#[serde(tag = "type", content = "value")]"#;

/// Predefined custom attributes for field annotations
const QUOTED: &str = r#"#[serde(with = "crate::serializers::from_str")]"#;
const QUOTED_WITH_DEFAULT: &str = r#"#[serde(with = "crate::serializers::from_str", default)]"#;
const DEFAULT: &str = r#"#[serde(default)]"#;
const HEXSTRING: &str = r#"#[serde(with = "crate::serializers::bytes::hexstring")]"#;
const BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::base64string")]"#;
const VEC_BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::vec_base64string")]"#;
const OPTIONAL: &str = r#"#[serde(with = "crate::serializers::optional")]"#;
const BYTES_SKIP_IF_EMPTY: &str = r#"#[serde(skip_serializing_if = "bytes::Bytes::is_empty")]"#;
const NULLABLEVECARRAY: &str = r#"#[serde(with = "crate::serializers::txs")]"#;
const NULLABLE: &str = r#"#[serde(with = "crate::serializers::nullable")]"#;
const ALIAS_POWER_QUOTED: &str =
    r#"#[serde(alias = "power", with = "crate::serializers::from_str")]"#;
const PART_SET_HEADER_TOTAL: &str =
    r#"#[serde(with = "crate::serializers::part_set_header_total")]"#;
const RENAME_EDPUBKEY: &str = r#"#[serde(rename = "tenderdash/PubKeyEd25519", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_SECPPUBKEY: &str = r#"#[serde(rename = "tenderdash/PubKeySecp256k1", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_SRPUBKEY: &str = r#"#[serde(rename = "tenderdash/PubKeySr25519", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_DUPLICATEVOTE: &str = r#"#[serde(rename = "tenderdash/DuplicateVoteEvidence")]"#;
const RENAME_LIGHTCLIENTATTACK: &str =
    r#"#[serde(rename = "tenderdash/LightClientAttackEvidence")]"#;
const EVIDENCE_VARIANT: &str = r#"#[serde(from = "crate::serializers::evidence::EvidenceVariant", into = "crate::serializers::evidence::EvidenceVariant")]"#;
const ALIAS_VALIDATOR_POWER_QUOTED: &str =
    r#"#[serde(alias = "ValidatorPower", with = "crate::serializers::from_str")]"#;
const ALIAS_TOTAL_VOTING_POWER_QUOTED: &str =
    r#"#[serde(alias = "TotalVotingPower", with = "crate::serializers::from_str")]"#;
const ALIAS_TIMESTAMP: &str = r#"#[serde(alias = "Timestamp")]"#;
const ALIAS_PARTS: &str = r#"#[serde(alias = "parts")]"#;

/// Custom type attributes applied on top of protobuf structs
/// The first item in the tuple defines the message where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".tenderdash.libs.bits.BitArray", SERIALIZED),
    (".tenderdash.types.EvidenceParams", SERIALIZED),
    (".tenderdash.types.BlockIDFlag", PRIMITIVE_ENUM),
    (".tenderdash.types.Block", SERIALIZED),
    (".tenderdash.types.Data", SERIALIZED),
    (".tenderdash.types.EvidenceList", SERIALIZED),
    (".tenderdash.types.Evidence", SERIALIZED),
    (".tenderdash.types.DuplicateVoteEvidence", SERIALIZED),
    (".tenderdash.types.Vote", SERIALIZED),
    (".tenderdash.types.BlockID", SERIALIZED),
    (".tenderdash.types.PartSetHeader", SERIALIZED),
    (".tenderdash.types.LightClientAttackEvidence", SERIALIZED),
    (".tenderdash.types.LightBlock", SERIALIZED),
    (".tenderdash.types.SignedHeader", SERIALIZED),
    (".tenderdash.types.Header", SERIALIZED),
    (".tenderdash.version.Consensus", SERIALIZED),
    (".tenderdash.types.Commit", SERIALIZED),
    (".tenderdash.types.CommitSig", SERIALIZED),
    (".tenderdash.types.ValidatorSet", SERIALIZED),
    (".tenderdash.crypto.PublicKey", SERIALIZED),
    (".tenderdash.crypto.PublicKey.sum", TYPE_TAG),
    (".tenderdash.types.Evidence.sum", TYPE_TAG),
    (".tenderdash.abci.ResponseInfo", SERIALIZED),
    (".tenderdash.types.CanonicalBlockID", SERIALIZED),
    (".tenderdash.types.CanonicalPartSetHeader", SERIALIZED),
    (".tenderdash.types.Validator", SERIALIZED),
    (".tenderdash.types.CanonicalVote", SERIALIZED),
    (".tenderdash.types.BlockMeta", SERIALIZED),
    (".tenderdash.types.Evidence", EVIDENCE_VARIANT),
    (".tenderdash.types.TxProof", SERIALIZED),
    (".tenderdash.crypto.Proof", SERIALIZED),
];

/// Custom field attributes applied on top of protobuf fields in (a) struct(s)
/// The first item in the tuple defines the field where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    (
        ".tenderdash.types.EvidenceParams.max_bytes",
        QUOTED_WITH_DEFAULT,
    ),
    (".tenderdash.version.Consensus.block", QUOTED),
    (".tenderdash.version.Consensus.app", QUOTED_WITH_DEFAULT),
    (".tenderdash.abci.ResponseInfo.data", DEFAULT),
    (".tenderdash.abci.ResponseInfo.version", DEFAULT),
    (
        ".tenderdash.abci.ResponseInfo.app_version",
        QUOTED_WITH_DEFAULT,
    ),
    (
        ".tenderdash.abci.ResponseInfo.last_block_height",
        QUOTED_WITH_DEFAULT,
    ),
    (".tenderdash.abci.ResponseInfo.last_block_app_hash", DEFAULT),
    (
        ".tenderdash.abci.ResponseInfo.last_block_app_hash",
        BYTES_SKIP_IF_EMPTY,
    ),
    (".tenderdash.types.BlockID.hash", HEXSTRING),
    (".tenderdash.types.BlockID.part_set_header", ALIAS_PARTS),
    (
        ".tenderdash.types.CanonicalBlockID.part_set_header",
        ALIAS_PARTS,
    ),
    (
        ".tenderdash.types.PartSetHeader.total",
        PART_SET_HEADER_TOTAL,
    ),
    (".tenderdash.types.PartSetHeader.hash", HEXSTRING),
    (".tenderdash.types.Header.height", QUOTED),
    (".tenderdash.types.Header.time", OPTIONAL),
    (".tenderdash.types.Header.last_commit_hash", HEXSTRING),
    (".tenderdash.types.Header.data_hash", HEXSTRING),
    (".tenderdash.types.Header.validators_hash", HEXSTRING),
    (".tenderdash.types.Header.next_validators_hash", HEXSTRING),
    (".tenderdash.types.Header.consensus_hash", HEXSTRING),
    (".tenderdash.types.Header.app_hash", HEXSTRING),
    (".tenderdash.types.Header.last_results_hash", HEXSTRING),
    (".tenderdash.types.Header.evidence_hash", HEXSTRING),
    (".tenderdash.types.Header.proposer_address", HEXSTRING),
    (".tenderdash.types.Data.txs", NULLABLEVECARRAY),
    (".tenderdash.types.EvidenceList.evidence", NULLABLE),
    (".tenderdash.types.Commit.height", QUOTED),
    (".tenderdash.types.Commit.signatures", NULLABLE),
    (".tenderdash.types.CommitSig.validator_address", HEXSTRING),
    (".tenderdash.types.CommitSig.timestamp", OPTIONAL),
    (".tenderdash.types.CommitSig.signature", BASE64STRING),
    (
        ".tenderdash.types.DuplicateVoteEvidence.total_voting_power",
        ALIAS_TOTAL_VOTING_POWER_QUOTED,
    ),
    (
        ".tenderdash.types.DuplicateVoteEvidence.validator_power",
        ALIAS_VALIDATOR_POWER_QUOTED,
    ),
    (
        ".tenderdash.types.DuplicateVoteEvidence.timestamp",
        ALIAS_TIMESTAMP,
    ),
    (".tenderdash.types.Vote.height", QUOTED),
    (".tenderdash.types.Vote.validator_address", HEXSTRING),
    (".tenderdash.types.Vote.signature", BASE64STRING),
    (".tenderdash.types.Vote.timestamp", OPTIONAL),
    (".tenderdash.types.Validator.address", HEXSTRING),
    (
        ".tenderdash.types.Validator.voting_power",
        ALIAS_POWER_QUOTED,
    ), // https://github.com/tendermint/tendermint/issues/5549
    (
        ".tenderdash.types.Validator.proposer_priority",
        QUOTED_WITH_DEFAULT,
    ), // Default is for /genesis deserialization
    (".tenderdash.types.BlockMeta.block_size", QUOTED),
    (".tenderdash.types.BlockMeta.num_txs", QUOTED),
    (".tenderdash.crypto.PublicKey.sum.ed25519", RENAME_EDPUBKEY),
    (
        ".tenderdash.crypto.PublicKey.sum.secp256k1",
        RENAME_SECPPUBKEY,
    ),
    (".tenderdash.crypto.PublicKey.sum.sr25519", RENAME_SRPUBKEY),
    (
        ".tenderdash.types.Evidence.sum.duplicate_vote_evidence",
        RENAME_DUPLICATEVOTE,
    ),
    (
        ".tenderdash.types.Evidence.sum.light_client_attack_evidence",
        RENAME_LIGHTCLIENTATTACK,
    ),
    (".tenderdash.types.TxProof.data", BASE64STRING),
    (".tenderdash.types.TxProof.root_hash", HEXSTRING),
    (".tenderdash.crypto.Proof.index", QUOTED),
    (".tenderdash.crypto.Proof.total", QUOTED),
    (".tenderdash.crypto.Proof.aunts", VEC_BASE64STRING),
    (".tenderdash.crypto.Proof.leaf_hash", BASE64STRING),
];
