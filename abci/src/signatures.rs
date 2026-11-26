//! Digital signature processing
//!
//! This module contains code for processing digital signatures, including
//! calculating message hash to be signed, and calculating signature digest.
//!
//! The code in this module is based on Tenderdash implementation.
//!
//! Two main traits are defined:
//! - [Signable] - for objects that can be signed/verified by Tenderdash.
//! - [Hashable] - for objects that can be serialized and hashed by Tenderdash.
//!
//! All [Signable] objects are also [Hashable], but not vice versa.
//! For example, [StateId] is [Hashable], but not [Signable], as it is only
//! part of some other signed objects. So is [Header]: its hash is the hash of
//! the block, which a [BlockId] carries.
//!
//! When signing or verifying signature, use [Signable::calculate_sign_hash] to
//! calculate signature digest and provide it as a digest directly to the
//! signature or verification function.

use std::{
    string::{String, ToString},
    vec::Vec,
};

use bytes::BufMut;

use crate::{
    Error,
    merkle::merkle_hash,
    proto::{
        prost::Message,
        types::{
            BlockId, CanonicalBlockId, CanonicalVote, CanonicalVoteExtension, Commit, Header,
            SignedMsgType, StateId, ValidatorSet, Vote, VoteExtension, VoteExtensionType,
        },
    },
};

const VOTE_REQUEST_ID_PREFIX: &str = "dpbvote";
const VOTE_EXTENSION_REQUEST_ID_PREFIX: &str = "dpevote";

/// Object that can be signed/verified by Tenderdash.
pub trait Signable: Hashable {
    #[deprecated = "replaced by calculate_sign_hash() to unify naming between core, platform and tenderdash"]
    fn sign_digest(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        self.calculate_sign_hash(chain_id, quorum_type, quorum_hash, height, round)
    }

    /// Returns message hash that should be provided directly to a
    /// signing/verification function.
    fn calculate_sign_hash(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error>;
}

impl Signable for Commit {
    fn calculate_sign_hash(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],

        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        if self.quorum_hash.ne(quorum_hash) {
            return Err(Error::Canonical("quorum hash mismatch".to_string()));
        }

        let request_id = sign_request_id(VOTE_REQUEST_ID_PREFIX, height, round);
        let sign_bytes_hash = self.calculate_msg_hash(chain_id, height, round)?;

        let digest = sign_hash(
            quorum_type,
            quorum_hash,
            request_id[..]
                .try_into()
                .expect("invalid request ID length"),
            &sign_bytes_hash,
        );

        // TODO: Remove once withdrawals are stable
        tracing::trace!(
            digest=hex::encode(&digest),
            ?quorum_type,
            quorum_hash=hex::encode(quorum_hash),
            request_id=hex::encode(request_id),
            commit=?self, "commit digest");

        Ok(digest)
    }
}

impl Signable for CanonicalVote {
    fn calculate_sign_hash(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],

        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        let request_id = sign_request_id(VOTE_REQUEST_ID_PREFIX, height, round);
        let sign_bytes_hash = self.calculate_msg_hash(chain_id, height, round)?;

        let digest = sign_hash(
            quorum_type,
            quorum_hash,
            request_id[..]
                .try_into()
                .expect("invalid request ID length"),
            &sign_bytes_hash,
        );

        // TODO: Remove once withdrawals are stable
        tracing::trace!(
            digest=hex::encode(&digest),
            ?quorum_type,
            quorum_hash=hex::encode(quorum_hash),
            request_id=hex::encode(request_id),
            vote=?self, "canonical vote digest");

        Ok(digest)
    }
}

impl Signable for VoteExtension {
    fn calculate_sign_hash(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        let (request_id, sign_bytes_hash) = match self.r#type() {
            VoteExtensionType::ThresholdRecover => {
                let request_id = sign_request_id(VOTE_EXTENSION_REQUEST_ID_PREFIX, height, round);
                let sign_bytes_hash = self.calculate_msg_hash(chain_id, height, round)?;

                (request_id, sign_bytes_hash)
            },

            VoteExtensionType::ThresholdRecoverRaw => {
                let mut sign_bytes_hash = self.extension.clone();
                sign_bytes_hash.reverse();

                let request_id = self.sign_request_id.clone().unwrap_or_default();
                let request_id = if request_id.is_empty() {
                    sign_request_id(VOTE_EXTENSION_REQUEST_ID_PREFIX, height, round)
                } else {
                    // we do double-sha256, and then reverse bytes
                    let mut request_id = lhash::sha256(&lhash::sha256(&request_id));
                    request_id.reverse();
                    request_id.to_vec()
                };

                (request_id, sign_bytes_hash)
            },

            VoteExtensionType::Default => unimplemented!(
                "vote extension of type {:?} cannot be signed",
                self.r#type()
            ),
        };
        let sign_hash = sign_hash(
            quorum_type,
            quorum_hash,
            request_id[..]
                .try_into()
                .expect("invalid request ID length"),
            &sign_bytes_hash,
        );

        // TODO: Remove once withdrawals are stable
        tracing::trace!(
            digest=hex::encode(&sign_hash),
            ?quorum_type,
            quorum_hash=hex::encode(quorum_hash),
            request_id=hex::encode(request_id),
            vote_extension=?self, "vote extension sign hash");

        Ok(sign_hash)
    }
}

fn sign_request_id(prefix: &str, height: i64, round: i32) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::from(prefix.as_bytes());
    buf.put_i64_le(height);
    buf.put_i32_le(round);

    lhash::sha256(&buf).to_vec()
}

fn sign_hash(
    quorum_type: u8,
    quorum_hash: &[u8; 32],
    request_id: &[u8; 32],
    sign_bytes_hash: &[u8],
) -> Vec<u8> {
    let mut quorum_hash = quorum_hash.to_vec();
    quorum_hash.reverse();

    let mut request_id = request_id.to_vec();
    request_id.reverse();

    let mut sign_bytes_hash = sign_bytes_hash.to_vec();
    sign_bytes_hash.reverse();

    let mut buf = Vec::<u8>::new();

    buf.put_u8(quorum_type);
    buf.append(&mut quorum_hash);
    buf.append(&mut request_id);
    buf.append(&mut sign_bytes_hash);

    let hash = lhash::sha256(&buf);
    // Note: In bls-signatures for go, we do double-hashing, so we need to also do
    // it here. See: https://github.com/dashpay/bls-signatures/blob/9329803969fd325dc0d5c9029ab15669d658ed5d/go-bindings/threshold.go#L62
    lhash::sha256(&hash).to_vec()
}

/// Calculate hash (sha256) of the data, using algorithms used by
/// Tenderdash.
pub trait Hashable {
    /// Generate hash of data to sign
    fn calculate_msg_hash(&self, chain_id: &str, height: i64, round: i32)
    -> Result<Vec<u8>, Error>;
}

impl<T: SignBytes> Hashable for T {
    /// Generate hash of data, to be used in signature process.
    ///
    /// Generates hash of the m
    fn calculate_msg_hash(
        &self,
        chain_id: &str,
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        let sb = self.sign_bytes(chain_id, height, round)?;
        let result = lhash::sha256(&sb);
        Ok(Vec::from(result))
    }
}

/// Marshals data into bytes to be used in signature process.
///
/// After marhaling, the bytes are hashed and then
trait SignBytes {
    /// Marshal into byte buffer, representing bytes to be used in signature
    /// process.
    ///
    /// See also: [SignDigest].
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error>;
}

impl SignBytes for StateId {
    fn sign_bytes(&self, _chain_id: &str, _height: i64, _round: i32) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.encode_length_delimited(&mut buf)
            .map_err(Error::Encode)?;

        Ok(buf.to_vec())
    }
}

impl SignBytes for BlockId {
    fn sign_bytes(&self, _chain_id: &str, _height: i64, _round: i32) -> Result<Vec<u8>, Error> {
        // determine if block id is zero
        if self.hash.is_empty()
            && (self.part_set_header.is_none()
                || self.part_set_header.as_ref().unwrap().hash.is_empty())
            && self.state_id.is_empty()
        {
            return Ok(Vec::<u8>::new());
        }

        let part_set_header = self.part_set_header.clone().unwrap_or_default();

        let block_id = CanonicalBlockId {
            hash: self.hash.clone(),
            part_set_header: Some(crate::proto::types::CanonicalPartSetHeader {
                total: part_set_header.total,
                hash: part_set_header.hash,
            }),
        };
        let mut buf = Vec::new();
        block_id
            .encode_length_delimited(&mut buf)
            .map_err(Error::Encode)?;

        Ok(buf)
    }
}

impl SignBytes for Vote {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        if height != self.height || round != self.round {
            return Err(Error::Canonical(String::from("vote height/round mismatch")));
        }

        let block_id = self
            .block_id
            .clone()
            .ok_or(Error::Canonical(String::from("missing vote.block id")))?;

        let block_id_hash = block_id.calculate_msg_hash(chain_id, height, round)?;
        let state_id_hash = block_id.state_id;

        let canonical = CanonicalVote {
            block_id: block_id_hash,
            state_id: state_id_hash,
            chain_id: chain_id.to_string(),
            height,
            round: round as i64,
            r#type: self.r#type,
        };

        canonical.sign_bytes(chain_id, height, round)
    }
}

impl SignBytes for Commit {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        if height != self.height || round != self.round {
            return Err(Error::Canonical(String::from(
                "commit height/round mismatch",
            )));
        }

        let block_id = self
            .block_id
            .clone()
            .ok_or(Error::Canonical(String::from("missing vote.block id")))?;

        let state_id_hash = block_id.state_id.clone();
        let block_id_hash = block_id.calculate_msg_hash(chain_id, height, round)?;

        let canonical = CanonicalVote {
            block_id: block_id_hash,
            state_id: state_id_hash,
            chain_id: chain_id.to_string(),
            height,
            round: round as i64,
            r#type: SignedMsgType::Precommit.into(),
        };

        canonical.sign_bytes(chain_id, height, round)
    }
}

impl SignBytes for CanonicalVote {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        if height != self.height || (round as i64) != self.round {
            return Err(Error::Canonical(String::from(
                "commit height/round mismatch",
            )));
        }

        // we just use some rough guesstimate of intial capacity for performance
        let mut buf = Vec::with_capacity(100);

        // Based on Tenderdash implementation in
        // https://github.com/dashpay/tenderdash/blob/bcb623bcf002ac54b26ed1324b98116872dd0da7/proto/tendermint/types/types.go#L56

        buf.put_i32_le(self.r#type().into()); // 4 bytes
        buf.put_i64_le(height); // 8 bytes
        buf.put_i64_le(round as i64); // 8 bytes

        buf.extend(&self.block_id); // 32 bytes
        buf.extend(&self.state_id); // 32 bytes
        if buf.len() != 4 + 8 + 8 + 32 + 32 {
            return Err(Error::Canonical(
                "cannot encode sign bytes: length of input data is invalid".to_string(),
            ));
        }
        buf.put(chain_id.as_bytes());

        // TODO: Remove once withdrawals are stable
        tracing::trace!(
            sign_bytes=hex::encode(&buf),
           height,round,
            vote=?self, "vote/commit sign bytes calculated");

        Ok(buf.to_vec())
    }
}

impl SignBytes for VoteExtension {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        match self.r#type() {
            VoteExtensionType::ThresholdRecover => {
                let ve = CanonicalVoteExtension {
                    chain_id: chain_id.to_string(),
                    extension: self.extension.clone(),
                    height,
                    round: round as i64,
                    r#type: self.r#type,
                };

                Ok(ve.encode_length_delimited_to_vec())
            },
            VoteExtensionType::ThresholdRecoverRaw => Ok(self.extension.to_vec()),
            _ => Err(Error::Canonical(format!(
                "unimplemented: vote extension of type {:?} cannot be signed",
                self.r#type()
            ))),
        }
    }
}

impl Hashable for ValidatorSet {
    /// Calculate hash of validator set.
    ///
    /// Calculates hash of validator set that is used in block header
    /// [Header.validators_hash](crate::proto::types::Header::validators_hash)
    /// and
    /// [Header.next_validators_hash](crate::proto::types::Header::next_validators_hash) fields.
    ///
    /// ## Tenderdash implementation
    ///
    /// Original Tenderdash implementation:
    /// ```go
    /// bzs := make([][]byte, 2)
    /// bzs[0] = vals.ThresholdPublicKey.Bytes()
    /// bzs[1] = vals.QuorumHash
    /// return merkle.HashFromByteSlices(bzs)
    /// ```
    ///
    /// ## Arguments
    ///
    /// All arguments are ignored.
    ///
    /// ## Returns
    ///
    /// Returns hash of validator set.
    fn calculate_msg_hash(
        &self,
        _chain_id: &str,
        _height: i64,
        _round: i32,
    ) -> Result<Vec<u8>, Error> {
        use tenderdash_proto::crypto::public_key::Sum::*;
        let threshold_public_key_enum = self
            .threshold_public_key
            .as_ref()
            .and_then(|key| key.sum.as_ref())
            .ok_or(Error::Canonical("missing threshold public key".to_string()))?;

        let threshold_public_key = match &threshold_public_key_enum {
            Bls12381(pk) => pk,
            Ed25519(pk) => pk,
            Secp256k1(pk) => pk,
        };

        let result = merkle_hash(&[threshold_public_key, &self.quorum_hash]);
        Ok(result.to_vec())
    }
}

impl Hashable for Header {
    /// Calculate hash of a block header, which is the hash of its block.
    ///
    /// This is the hash that [BlockId.hash](crate::proto::types::BlockId::hash)
    /// carries and, through the block ID, the one a commit signs.
    ///
    /// The core chain locked height does not change this hash: the
    /// [StateId] is what commits to it.
    ///
    /// ## Tenderdash implementation
    ///
    /// The hash is the Merkle root of the header's fields, each in its
    /// protobuf encoding. A string, number or hash is wrapped in a one-field
    /// message first (`cdcEncode`), so one that is empty or zero is an empty
    /// leaf. `cdcEncode` does not support the type of the core chain locked
    /// height, so that field is always an empty leaf:
    ///
    /// ```go
    /// return merkle.HashFromByteSlices([][]byte{
    ///     version,
    ///     cdcEncode(h.ChainID),
    ///     cdcEncode(h.Height),
    ///     cdcEncode(h.CoreChainLockedHeight),
    ///     blockTime,
    ///     lastBlockID,
    ///     cdcEncode(h.LastCommitHash),
    ///     cdcEncode(h.DataHash),
    ///     cdcEncode(h.ValidatorsHash),
    ///     cdcEncode(h.NextValidatorsHash),
    ///     cdcEncode(h.ConsensusHash),
    ///     cdcEncode(h.NextConsensusHash),
    ///     cdcEncode(h.AppHash),
    ///     cdcEncode(h.ResultsHash),
    ///     cdcEncode(h.EvidenceHash),
    ///     cdcEncode(h.ProposerProTxHash),
    ///     cdcEncode(h.ProposedAppVersion),
    /// })
    /// ```
    ///
    /// ## Arguments
    ///
    /// All arguments are ignored.
    ///
    /// ## Returns
    ///
    /// Returns hash of the header. Fails for a header without a validators
    /// hash, which Tenderdash gives no hash, and for one without a time.
    fn calculate_msg_hash(
        &self,
        _chain_id: &str,
        _height: i64,
        _round: i32,
    ) -> Result<Vec<u8>, Error> {
        if self.validators_hash.is_empty() {
            return Err(Error::Canonical("missing validators hash".to_string()));
        }
        let time = self
            .time
            .as_ref()
            .ok_or(Error::Canonical("missing header time".to_string()))?;

        // Tenderdash always encodes the part set header of a block ID, also
        // when the header has no last block
        let last_block_id = self.last_block_id.clone().unwrap_or_default();
        let last_block_id = BlockId {
            part_set_header: Some(last_block_id.part_set_header.unwrap_or_default()),
            ..last_block_id
        };

        let result = merkle_hash(&[
            self.version.unwrap_or_default().encode_to_vec(),
            self.chain_id.encode_to_vec(),
            self.height.encode_to_vec(),
            // core chain locked height
            Vec::new(),
            time.encode_to_vec(),
            last_block_id.encode_to_vec(),
            self.last_commit_hash.encode_to_vec(),
            self.data_hash.encode_to_vec(),
            self.validators_hash.encode_to_vec(),
            self.next_validators_hash.encode_to_vec(),
            self.consensus_hash.encode_to_vec(),
            self.next_consensus_hash.encode_to_vec(),
            self.app_hash.encode_to_vec(),
            self.results_hash.encode_to_vec(),
            self.evidence_hash.encode_to_vec(),
            self.proposer_pro_tx_hash.encode_to_vec(),
            self.proposed_app_version.encode_to_vec(),
        ]);

        Ok(result.to_vec())
    }
}

#[cfg(test)]
pub mod tests {
    use std::{string::ToString, vec::Vec};

    use super::SignBytes;
    use crate::{
        Error,
        proto::{
            google::protobuf::Timestamp,
            types::{
                BlockId, Commit, Header, PartSetHeader, SignedMsgType, StateId, Vote,
                VoteExtension, VoteExtensionType,
            },
            version::Consensus,
        },
        signatures::{Hashable, Signable},
    };

    #[test]
    /// Compare sign bytes for Vote with sign bytes generated by Tenderdash and
    /// put into `expect_sign_bytes`.
    fn vote_sign_bytes() {
        let h = [1u8, 2, 3, 4].repeat(8);

        let state_id_hash =
            hex::decode("d7509905b5407ee72dadd93b4ae70a24ad8a7755fc677acd2b215710a05cfc47")
                .unwrap();
        let expect_sign_bytes = hex::decode("0100000001000000000000000200000000000000fb\
                7c89bf010a91d50f890455582b7fed0c346e53ab33df7da0bcd85c10fa92ead7509905b5407ee72dadd93b\
                4ae70a24ad8a7755fc677acd2b215710a05cfc47736f6d652d636861696e")
        .unwrap();

        let vote = Vote {
            r#type: SignedMsgType::Prevote as i32,
            height: 1,
            round: 2,
            block_id: Some(crate::proto::types::BlockId {
                hash: h.clone(),
                part_set_header: Some(PartSetHeader {
                    total: 1,
                    hash: h.clone(),
                }),
                state_id: state_id_hash,
            }),
            ..Default::default()
        };
        let chain_id = "some-chain".to_string();
        let height = vote.height;
        let round = vote.round;

        let actual = vote.sign_bytes(&chain_id, height, round).unwrap();

        assert_eq!(expect_sign_bytes, actual);
    }

    #[test]
    fn commit_sign_bytes() {
        let h = [1u8, 2, 3, 4].repeat(8);

        let state_id_hash =
            hex::decode("d7509905b5407ee72dadd93b4ae70a24ad8a7755fc677acd2b215710a05cfc47")
                .unwrap();
        let expect_sign_bytes = hex::decode("0200000001000000000000000200000000000000fb7c89bf010a91d5\
            0f890455582b7fed0c346e53ab33df7da0bcd85c10fa92ead7509905b5407ee72dadd93b4ae70a24ad8a7755fc677acd2b215710\
            a05cfc47736f6d652d636861696e")
        .unwrap();

        let commit = Commit {
            height: 1,
            round: 2,
            block_id: Some(crate::proto::types::BlockId {
                hash: h.clone(),
                part_set_header: Some(PartSetHeader {
                    total: 1,
                    hash: h.clone(),
                }),
                state_id: state_id_hash,
            }),
            ..Default::default()
        };
        let chain_id = "some-chain".to_string();
        let height = commit.height;
        let round = commit.round;

        let actual = commit.sign_bytes(&chain_id, height, round).unwrap();

        assert_eq!(expect_sign_bytes, actual);
    }

    #[test]
    fn vote_extension_threshold_sign_bytes() {
        let ve = VoteExtension {
            extension: Vec::from([1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8]),
            r#type: VoteExtensionType::ThresholdRecover.into(),
            signature: Default::default(),
            sign_request_id: None,
        };

        let chain_id = "some-chain".to_string();
        let height = 1;
        let round = 2;

        let expect_sign_bytes = hex::decode(
            "2a0a080102030405060708110100000000000000190200000000000000220a736f6d652d636861696e2801",
        )
        .unwrap();

        let actual = ve.sign_bytes(&chain_id, height, round).unwrap();

        assert_eq!(expect_sign_bytes, actual);
    }

    /// test vector for threshold-raw vote extensions
    ///
    /// Returns expected sig hash and vote extension
    fn ve_threshold_raw() -> ([u8; 32], VoteExtension) {
        let ve = VoteExtension {
            extension: [1, 2, 3, 4, 5, 6, 7, 8].repeat(4),
            r#type: VoteExtensionType::ThresholdRecoverRaw.into(),
            signature: Default::default(),
            sign_request_id: Some("dpevote-someSignRequestID".as_bytes().to_vec()),
        };
        let expected_sign_hash: [u8; 32] = [
            0xe, 0x88, 0x8d, 0xa8, 0x97, 0xf1, 0xc0, 0xfd, 0x6a, 0xe8, 0x3b, 0x77, 0x9b, 0x5, 0xdd,
            0x28, 0xc, 0xe2, 0x58, 0xf6, 0x4c, 0x86, 0x1, 0x34, 0xfa, 0x4, 0x27, 0xe1, 0xaa, 0xab,
            0x1a, 0xde,
        ];

        (expected_sign_hash, ve)
    }

    #[test]
    fn test_ve_threshold_raw_sign_bytes() {
        let (_, ve) = ve_threshold_raw();
        let expected_sign_bytes = ve.extension.clone();

        // chain_id, height and round are unused
        let chain_id = String::new();
        let height = -1;
        let round = -1;

        let actual = ve.sign_bytes(&chain_id, height, round).unwrap();

        assert_eq!(expected_sign_bytes, actual);
    }

    #[test]
    fn test_sign_digest() {
        let quorum_hash: [u8; 32] =
            hex::decode("6A12D9CF7091D69072E254B297AEF15997093E480FDE295E09A7DE73B31CEEDD")
                .unwrap()
                .try_into()
                .unwrap();

        let request_id = super::sign_request_id(super::VOTE_REQUEST_ID_PREFIX, 1001, 0);
        let request_id = request_id[..].try_into().unwrap();

        let sign_bytes_hash =
            hex::decode("0CA3D5F42BDFED0C4FDE7E6DE0F046CC76CDA6CEE734D65E8B2EE0E375D4C57D")
                .unwrap();

        let expect_sign_hash =
            hex::decode("DA25B746781DDF47B5D736F30B1D9D0CC86981EEC67CBE255265C4361DEF8C2E")
                .unwrap();

        let sign_hash = super::sign_hash(100, &quorum_hash, request_id, &sign_bytes_hash);
        assert_eq!(expect_sign_hash, sign_hash); // 194,4
    }

    #[test]
    fn test_ve_threshold_raw_sign_digest() {
        const QUORUM_TYPE: u8 = 106;
        let quorum_hash: [u8; 32] = [8u8, 7, 6, 5, 4, 3, 2, 1]
            .repeat(4)
            .try_into()
            .expect("invalid quorum hash length");
        let (expected_sign_hash, ve) = ve_threshold_raw();

        // height, round, chain id are not used in sign digest for threshold-raw
        let sign_hash = ve
            .calculate_sign_hash("", QUORUM_TYPE, &quorum_hash, -1, -1)
            .expect("sign digest failed");

        assert_eq!(sign_hash, expected_sign_hash);
    }

    #[test]
    fn test_validator_set_hash() {
        use crate::proto::crypto::{PublicKey, public_key::Sum::Bls12381};

        const QUORUM_HASH_HEX: &str =
            "703ee5bfc78765cc9e151d8dd84e30e196ababa83ac6cbdee31a88a46bba81b9";
        const THRESHOLD_PUB_KEY_HEX: &str = "830e45e45e6414d9d615473cc2814e6b171c508f9c77e8b16924b74594f61c9956a6fa16335e98467eac8d8bdb76d187";
        const VALIDATORS_HASH_HEX: &str =
            "81742F95E99EAE96ABC727FE792CECB4996205DE6BFC88AFEE1F60B96BC648B2";

        let pubkey_vec = hex::decode(THRESHOLD_PUB_KEY_HEX).unwrap();
        let quorum_hash = hex::decode(QUORUM_HASH_HEX).unwrap();
        let expected_validators_hash = hex::decode(VALIDATORS_HASH_HEX).unwrap();

        let threshold_public_key = PublicKey {
            sum: Some(Bls12381(pubkey_vec)),
        };

        let vs = crate::proto::types::ValidatorSet {
            threshold_public_key: Some(threshold_public_key),
            quorum_hash,
            ..Default::default()
        };

        // chain_id, height and round are unused
        let actual = vs.calculate_msg_hash("", 0, 0).unwrap();

        assert_eq!(expected_validators_hash, actual,);
    }

    /// The header of the "Generates expected hash" case of Tenderdash test
    /// [TestHeaderHash](https://github.com/dashpay/tenderdash/blob/v2.0.0-dev.1/types/block_test.go#L370),
    /// which sets every field.
    fn tenderdash_header() -> Header {
        let checksum = |seed: &str| lhash::sha256(seed.as_bytes()).to_vec();
        let last_state_id = StateId {
            app_version: 1,
            height: 3,
            app_hash: checksum("app_hash"),
            core_chain_locked_height: 1,
            // 2022-03-04T05:06:07.000000008Z, in milliseconds
            time: 1_646_370_367_000,
        };

        Header {
            version: Some(Consensus { block: 1, app: 2 }),
            chain_id: "chainId".to_string(),
            height: 3,
            // 2019-10-13T16:14:44Z
            time: Some(Timestamp {
                seconds: 1_570_983_284,
                nanos: 0,
            }),
            last_block_id: Some(BlockId {
                hash: [0u8; 32].to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 6,
                    hash: [0u8; 32].to_vec(),
                }),
                state_id: last_state_id.calculate_msg_hash("", 0, 0).unwrap(),
            }),
            last_commit_hash: checksum("last_commit_hash"),
            data_hash: checksum("data_hash"),
            validators_hash: checksum("validators_hash"),
            next_validators_hash: checksum("next_validators_hash"),
            consensus_hash: checksum("consensus_hash"),
            next_consensus_hash: checksum("next_consensus_hash"),
            app_hash: checksum("app_hash"),
            results_hash: checksum("last_results_hash"),
            evidence_hash: checksum("evidence_hash"),
            proposed_app_version: 1,
            proposer_pro_tx_hash: checksum("proposer_pro_tx_hash"),
            core_chain_locked_height: 1,
        }
    }

    fn header_hash(header: &Header) -> String {
        // chain_id, height and round are unused
        hex::encode(header.calculate_msg_hash("", 0, 0).unwrap())
    }

    #[test]
    fn test_header_hash() {
        assert_eq!(
            header_hash(&tenderdash_header()),
            "ff24ddab9e1550beb40ab7ad432a4d577560e9b87a80c7bb86e75263974b87e0"
        );
    }

    /// Tenderdash test
    /// [TestHeaderHashVector](https://github.com/dashpay/tenderdash/blob/v2.0.0-dev.1/types/block_test.go#L1419):
    /// no last block, a time before 1970 with nanoseconds, and hashes that
    /// are the text of a hex string.
    #[test]
    fn test_header_hash_vector() {
        let text = b"f2564c78071e26643ae9b3e2a19fa0dc10d4d9e873aa0be808660123f11a1e78".to_vec();
        let mut header = Header {
            version: Some(Consensus { block: 1, app: 1 }),
            chain_id: "test".to_string(),
            height: 50,
            // what Go's `time.Date(math.MaxInt64, 0, 0, 0, 0, 0, math.MaxInt64,
            // time.UTC)` wraps around to: 0291-03-10T13:13:40.854775807Z
            time: Some(Timestamp {
                seconds: -52_978_185_980,
                nanos: 854_775_807,
            }),
            last_block_id: Some(BlockId::default()),
            last_commit_hash: text.clone(),
            data_hash: text.clone(),
            validators_hash: text.clone(),
            next_validators_hash: text.clone(),
            consensus_hash: text.clone(),
            next_consensus_hash: text.clone(),
            app_hash: text.clone(),
            results_hash: text.clone(),
            evidence_hash: text.clone(),
            proposer_pro_tx_hash: text,
            ..Default::default()
        };
        const EXPECTED: &str = "1476e858bfe231d7cd767a9693f63fe6a8757e8c07aba432f338a5ecba2b0342";

        assert_eq!(header_hash(&header), EXPECTED);

        // an absent last block ID is an empty one
        header.last_block_id = None;
        assert_eq!(header_hash(&header), EXPECTED);
    }

    /// Every field changes the hash, except the core chain locked height.
    #[test]
    fn test_header_hash_covers_every_field_but_core_chain_locked_height() {
        type Change = (&'static str, fn(&mut Header));
        let signed = tenderdash_header();
        let hash = header_hash(&signed);
        let changes: [Change; 18] = [
            ("version.block", |h| h.version.as_mut().unwrap().block += 1),
            ("version.app", |h| h.version.as_mut().unwrap().app += 1),
            ("chain_id", |h| h.chain_id.push('x')),
            ("height", |h| h.height += 1),
            ("time.seconds", |h| h.time.as_mut().unwrap().seconds += 1),
            ("time.nanos", |h| h.time.as_mut().unwrap().nanos += 1),
            ("last_block_id", |h| {
                h.last_block_id.as_mut().unwrap().state_id[0] ^= 1
            }),
            ("last_commit_hash", |h| h.last_commit_hash[0] ^= 1),
            ("data_hash", |h| h.data_hash[0] ^= 1),
            ("validators_hash", |h| h.validators_hash[0] ^= 1),
            ("next_validators_hash", |h| h.next_validators_hash[0] ^= 1),
            ("consensus_hash", |h| h.consensus_hash[0] ^= 1),
            ("next_consensus_hash", |h| h.next_consensus_hash[0] ^= 1),
            ("app_hash", |h| h.app_hash[0] ^= 1),
            ("results_hash", |h| h.results_hash[0] ^= 1),
            ("evidence_hash", |h| h.evidence_hash[0] ^= 1),
            ("proposed_app_version", |h| h.proposed_app_version += 1),
            ("proposer_pro_tx_hash", |h| h.proposer_pro_tx_hash[0] ^= 1),
        ];

        for (field, change) in changes {
            let mut header = signed.clone();
            change(&mut header);
            assert_ne!(header_hash(&header), hash, "{field}");
        }

        let mut header = signed.clone();
        header.core_chain_locked_height += 1;
        assert_eq!(header_hash(&header), hash);
    }

    /// Tenderdash gives a header without a validators hash no hash at all.
    #[test]
    fn test_header_hash_needs_validators_hash_and_time() {
        let mut header = tenderdash_header();
        header.validators_hash = Vec::new();
        assert!(matches!(
            header.calculate_msg_hash("", 0, 0),
            Err(Error::Canonical(_))
        ));

        let mut header = tenderdash_header();
        header.time = None;
        assert!(matches!(
            header.calculate_msg_hash("", 0, 0),
            Err(Error::Canonical(_))
        ));
    }
}
