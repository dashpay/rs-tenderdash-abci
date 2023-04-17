//! Digital signature processing
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use bytes::BufMut;
use prost::Message;

use crate::{
    types::{
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
        quorum_hash: Vec<u8>,
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error>;
}

impl SignDigest for Commit {
    fn sign_digest(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: Vec<u8>,

        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        if self.quorum_hash != quorum_hash {
            return Err(Error::create_canonical("quorum hash mismatch".to_string()));
        }

        let request_id = sign_request_id(VOTE_REQUEST_ID_PREFIX, height, round);
        let sign_bytes_hash = self.sha256(chain_id, height, round)?;

        Ok(sign_digest(
            quorum_type,
            quorum_hash,
            request_id,
            sign_bytes_hash,
        ))
    }
}

impl SignDigest for VoteExtension {
    fn sign_digest(
        &self,
        chain_id: &str,
        quorum_type: u8,
        quorum_hash: Vec<u8>,
        height: i64,
        round: i32,
    ) -> Result<Vec<u8>, Error> {
        let request_id = sign_request_id(VOTE_EXTENSION_REQUEST_ID_PREFIX, height, round);
        let sign_bytes_hash = self.sha256(chain_id, height, round)?;

        Ok(sign_digest(
            quorum_type,
            quorum_hash,
            request_id,
            sign_bytes_hash,
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
    mut quorum_hash: Vec<u8>,
    mut request_id: Vec<u8>,
    mut sign_bytes_hash: Vec<u8>,
) -> Vec<u8> {
    quorum_hash.reverse();
    request_id.reverse();
    sign_bytes_hash.reverse();

    let mut buf = Vec::<u8>::new();

    buf.put_u8(quorum_type);
    buf.append(&mut quorum_hash);
    buf.append(&mut request_id);
    buf.append(&mut sign_bytes_hash);

    let hash = lhash::sha256(&buf);
    // FIXME: In bls-signatures for go, we do double-hashing, so we need to also do
    // it here. See https://github.com/dashpay/bls-signatures/blob/main/go-bindings/threshold.go#L62
    lhash::sha256(&hash).to_vec()
}

trait SignBytes {
    /// Marshal into byte buffer, representing bytes to be used in signature
    /// process.
    ///
    /// See also: [SignDigest].
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error>;

    /// Generate hash of data to sign
    fn sha256(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        // todo!()
        let sb = self.sign_bytes(chain_id, height, round)?;
        let result = lhash::sha256(&sb);
        Ok(Vec::from(result))
    }
}

impl SignBytes for StateId {
    fn sign_bytes(&self, _chain_id: &str, _height: i64, _round: i32) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.encode_length_delimited(&mut buf)
            .map_err(Error::encode_message)?;

        Ok(buf.to_vec())
    }
}

impl SignBytes for BlockId {
    fn sign_bytes(&self, _chain_id: &str, _height: i64, _round: i32) -> Result<Vec<u8>, Error> {
        let part_set_header = self.part_set_header.clone().unwrap_or_default();

        let block_id = CanonicalBlockId {
            hash: self.hash.clone(),
            part_set_header: Some(crate::types::CanonicalPartSetHeader {
                total: part_set_header.total,
                hash: part_set_header.hash,
            }),
        };
        let mut buf = Vec::new();
        block_id
            .encode_length_delimited(&mut buf)
            .map_err(Error::encode_message)?;

        Ok(buf)
    }
}

impl SignBytes for Vote {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        if height != self.height || round != self.round {
            return Err(Error::create_canonical(String::from(
                "vote height/round mismatch",
            )));
        }

        let block_id = self
            .block_id
            .clone()
            .ok_or(Error::create_canonical(String::from(
                "missing vote.block id",
            )))?;

        vote_sign_bytes(block_id, self.r#type(), chain_id, height, round)
    }
}

impl SignBytes for Commit {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        if height != self.height || round != self.round {
            return Err(Error::create_canonical(String::from(
                "commit height/round mismatch",
            )));
        }

        let block_id = self
            .block_id
            .clone()
            .ok_or(Error::create_canonical(String::from(
                "missing vote.block id",
            )))?;

        vote_sign_bytes(block_id, SignedMsgType::Precommit, chain_id, height, round)
    }
}

impl SignBytes for VoteExtension {
    fn sign_bytes(&self, chain_id: &str, height: i64, round: i32) -> Result<Vec<u8>, Error> {
        if self.r#type() != VoteExtensionType::ThresholdRecover {
            return Err(Error::create_canonical(String::from(
                "only ThresholdRecover vote extensions can be signed",
            )));
        }
        let ve = CanonicalVoteExtension {
            chain_id: chain_id.to_string(),
            extension: self.extension.clone(),
            height,
            round: round as i64,
            r#type: self.r#type,
        };

        Ok(ve.encode_length_delimited_to_vec())
    }
}

/// Generate sign bytes for a vote / commit
///
/// See https://github.com/dashpay/tenderdash/blob/bcb623bcf002ac54b26ed1324b98116872dd0da7/proto/tendermint/types/types.go#L56
fn vote_sign_bytes(
    block_id: BlockId,
    vote_type: SignedMsgType,
    chain_id: &str,
    height: i64,
    round: i32,
) -> Result<Vec<u8>, Error> {
    // we just use some rough guesstimate of intial capacity
    let mut buf = Vec::with_capacity(80);

    let state_id = block_id.state_id.clone();
    let block_id = block_id.sha256(chain_id, height, round)?;

    buf.put_i32_le(vote_type.into());
    buf.put_i64_le(height);
    buf.put_i64_le(round as i64);

    buf.extend(block_id);
    buf.extend(state_id);
    buf.put(chain_id.as_bytes());

    Ok(buf.to_vec())
}

#[cfg(test)]
pub mod tests {
    use alloc::{string::ToString, vec::Vec};

    use super::SignBytes;
    use crate::types::{
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
            block_id: Some(crate::types::BlockId {
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
            block_id: Some(crate::types::BlockId {
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
    fn vote_extension_sign_bytes() {
        let ve = VoteExtension {
            extension: Vec::from([1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8]),
            r#type: VoteExtensionType::ThresholdRecover.into(),
            signature: Default::default(),
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
}
