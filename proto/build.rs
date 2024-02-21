use std::env;

fn main() {
    const DEFAULT_VERSION: &str = "v0.14.0-dev.2";

    // check if TENDERDASH_COMMITISH is already set; if not, set it to the current
    // version
    let commitish = env::var("TENDERDASH_COMMITISH").unwrap_or_default();
    if commitish.is_empty() {
        env::set_var("TENDERDASH_COMMITISH", DEFAULT_VERSION);
    }

    tenderdash_proto_compiler::proto_compile();

    println!("cargo:rerun-if-changed=../proto-compiler/src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=TENDERDASH_COMMITISH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_STD");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_GRPC_SERVER");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_GRPC_CLIENT");
}
