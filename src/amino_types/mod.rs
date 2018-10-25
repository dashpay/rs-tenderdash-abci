//! Message types serialized using the Amino serialization format
//! <https://github.com/tendermint/amino_rs>

#![allow(missing_docs)]

pub mod block_id;
pub mod ed25519;
pub mod heartbeat;
pub mod ping;
pub mod poisonpill;
pub mod proposal;
pub mod remote_error;
pub mod secret_connection;
pub mod signature;
pub mod time;
pub mod vote;

pub use self::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader, PartsSetHeader},
    ed25519::{PubKeyMsg, AMINO_NAME as PUBKEY_AMINO_NAME},
    heartbeat::{
        SignHeartbeatRequest, SignedHeartbeatResponse, AMINO_NAME as HEARTBEAT_AMINO_NAME,
    },
    ping::{PingRequest, PingResponse, AMINO_NAME as PING_AMINO_NAME},
    poisonpill::{PoisonPillMsg, AMINO_NAME as POISON_PILL_AMINO_NAME},
    proposal::{SignProposalRequest, SignedProposalResponse, AMINO_NAME as PROPOSAL_AMINO_NAME},
    remote_error::RemoteError,
    secret_connection::AuthSigMessage,
    signature::{SignableMsg, SignedMsgType},
    time::TimeMsg,
    vote::{SignVoteRequest, SignedVoteResponse, AMINO_NAME as VOTE_AMINO_NAME},
};
