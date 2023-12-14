//! Digital signature processing
//!
//! The workflow is as follows:
//!
//! 1. First, we serialize data to get bytes to be signed using
//! [SignBytes::sign_bytes].
//!
//! 2. Then, we calculate hash with [SignBytes::sha256].
//!
//! 3. Then, we calculate digest using [SignDigest::sign_digest] that is passed
//! directly to the public/private key.

use std::{
    string::{String, ToString},
    vec::Vec,
};

use bytes::BufMut;
use prost::Message;
use tenderdash_proto::types::CanonicalVote;

use crate::{
    proto::types::{
        BlockId, CanonicalBlockId, CanonicalVoteExtension, Commit, SignedMsgType, StateId, Vote,
        VoteExtension, VoteExtensionType,
    },
    Error,
};

const VOTE_REQUEST_ID_PREFIX: &str = "dpbvote";
const VOTE_EXTENSION_REQUEST_ID_PREFIX: &str = "dpevote";

/// SignDigest returns message digest that should be provided directly to a
/// signing/verification function (aka Sign ID)
pub trait SignDigest {
    fn sign_digest(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error>;
}

impl SignDigest for Commit {
    fn sign_digest(
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
        let sign_bytes_hash = self.sha256(chain_id, height, round)?;

        let digest = sign_digest(
            quorum_type,
            quorum_hash,
            request_id[..]
                .try_into()
                .expect("invalid request ID length"),
            &sign_bytes_hash,
        );

        tracing::trace!(
            digest=hex::encode(&digest),
            ?quorum_type,
            quorum_hash=hex::encode(quorum_hash),
            request_id=hex::encode(request_id),
            commit=?self, "commit digest");

        Ok(digest)
    }
}

impl SignDigest for CanonicalVote {
    fn sign_digest(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],

        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        let request_id = sign_request_id(VOTE_REQUEST_ID_PREFIX, height, round);
        let sign_bytes_hash = self.sha256(chain_id, height, round)?;

        let digest = sign_digest(
            quorum_type,
            quorum_hash,
            request_id[..]
                .try_into()
                .expect("invalid request ID length"),
            &sign_bytes_hash,
        );

        tracing::trace!(
            digest=hex::encode(&digest),
            ?quorum_type,
            quorum_hash=hex::encode(quorum_hash),
            request_id=hex::encode(request_id),
            vote=?self, "canonical vote digest");

        Ok(digest)
    }
}

impl SignDigest for VoteExtension {
    fn sign_digest(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: &[u8; 32],
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        let request_id = sign_request_id(VOTE_EXTENSION_REQUEST_ID_PREFIX, height, round);
        let sign_bytes_hash = self.sha256(chain_id, height, round)?;

        Ok(sign_digest(
            quorum_type,
            quorum_hash,
            request_id[..].try_into().unwrap(),
            &sign_bytes_hash,
        ))
    }
}

fn sign_request_id(prefix: &str, height: i64, round: i32) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::from(prefix.as_bytes());
    buf.put_i64_le(height);
    buf.put_i32_le(round);

    lhash::sha256(&buf).to_vec()
}

fn sign_digest(
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

pub trait SignBytes {
    /// Marshal into byte buffer, representing bytes to be used in signature
    /// process.
    ///
    /// See also: [SignDigest].
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error>;

    /// Generate hash of data to sign
    fn sha256(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        let sb = self.sign_bytes(chain_id, height, round)?;
        let result = lhash::sha256(&sb);
        Ok(Vec::from(result))
    }
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

        let block_id_hash = block_id.sha256(chain_id, height, round)?;
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
        let block_id_hash = block_id.sha256(chain_id, height, round)?;

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

#[cfg(test)]
pub mod tests {
    use std::{string::ToString, vec::Vec};

    use super::SignBytes;
    use crate::proto::types::{
        Commit, PartSetHeader, SignedMsgType, Vote, VoteExtension, VoteExtensionType,
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

    #[test]
    fn vote_extension_threshold_raw_sign_bytes() {
        const EXTENSION: &[u8] = &[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];
        let ve = VoteExtension {
            extension: EXTENSION.to_vec(),
            r#type: VoteExtensionType::ThresholdRecoverRaw.into(),
            signature: Default::default(),
            sign_request_id: Some("dpe-sign-request-id".as_bytes().to_vec()),
        };

        let chain_id = "some-chain".to_string();
        let height = 1;
        let round = 2;

        let expect_sign_bytes = EXTENSION.to_vec();

        let actual = ve.sign_bytes(&chain_id, height, round).unwrap();

        assert_eq!(expect_sign_bytes, actual);
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

        let expect_sign_id =
            hex::decode("DA25B746781DDF47B5D736F30B1D9D0CC86981EEC67CBE255265C4361DEF8C2E")
                .unwrap();

        let sign_id = super::sign_digest(100, &quorum_hash, request_id, &sign_bytes_hash);
        assert_eq!(expect_sign_id, sign_id); // 194,4
    }

    #[test]
    fn test_raw_extension_sign_digest() {
        const QUORUM_TYPE: u8 = 106;

        let quorum_hash: [u8; 32] =
            hex::decode("dddabfe1c883dd8a2c71c4281a4212c3715a61f87d62a99aaed0f65a0506c053")
                .unwrap()
                .try_into()
                .unwrap();

        let request_id =
            hex::decode("922a8fc39b6e265ca761eaaf863387a5e2019f4795a42260805f5562699fd9fa")
                .unwrap();
        let request_id = request_id[..].try_into().unwrap();

        let sign_bytes_hash =
            hex::decode("7dfb2432d37f004c4eb2b9aebf601ba4ad59889b81d2e8c7029dce3e0bf8381c")
                .unwrap();

        let mut expect_sign_id =
            hex::decode("6d98f773cef8484432c4946c6b96e04aab39fd119c77de2f21d668dd17d5d2f6")
                .unwrap();
        expect_sign_id.reverse();

        let sign_id = super::sign_digest(QUORUM_TYPE, &quorum_hash, request_id, &sign_bytes_hash);
        assert_eq!(expect_sign_id, sign_id);
    }
}
