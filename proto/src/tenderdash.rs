//! Tenderdash-proto auto-generated sub-modules for Tenderdash

pub mod mempool {
    include!("prost/tendermint.mempool.rs");
}

pub mod blocksync {
    include!("prost/tendermint.blocksync.rs");
}

pub mod version {
    include!("prost/tendermint.version.rs");
}

pub mod libs {
    pub mod bits {
        include!("prost/tendermint.libs.bits.rs");
    }
}

pub mod abci {
    include!("prost/tendermint.abci.rs");
}

pub mod crypto {
    include!("prost/tendermint.crypto.rs");
}

pub mod consensus {
    include!("prost/tendermint.consensus.rs");
}

pub mod p2p {
    include!("prost/tendermint.p2p.rs");
}

pub mod types {
    include!("prost/tendermint.types.rs");
}

pub mod privval {
    include!("prost/tendermint.privval.rs");
}

pub mod state {
    include!("prost/tendermint.state.rs");
}

pub mod statesync {
    include!("prost/tendermint.statesync.rs");
}

pub mod meta {
    pub const REPOSITORY: &str = "https://github.com/dashpay/tenderdash";
    pub const COMMITISH: &str = "v0.10-dev";
}
