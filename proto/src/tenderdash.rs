//! Tenderdash-proto auto-generated sub-modules for Tenderdash

pub mod abci {
    include!("prost/tendermint.abci.rs");
}

pub mod crypto {
    include!("prost/tendermint.crypto.rs");
}

pub mod types {
    include!("prost/tendermint.types.rs");
}

pub mod version {
    include!("prost/tendermint.version.rs");
}

pub mod meta {
    pub const REPOSITORY: &str = "https://github.com/dashpay/tenderdash";
    pub const COMMITISH: &str = "v0.10-dev";
}
