use std::env;

fn main() {
    // default Tenderdash version to use if TENDERDASH_COMMITISH is not set
    const DEFAULT_VERSION: &str = "v0.14.0-dev.5";

    // check if TENDERDASH_COMMITISH is already set; if not, set it to the current
    // version
    let commitish = env::var("TENDERDASH_COMMITISH").unwrap_or_default();
    if commitish.is_empty() {
        env::set_var("TENDERDASH_COMMITISH", DEFAULT_VERSION);
    }

    #[cfg(feature = "std")]
    tenderdash_proto_compiler::proto_compile("tenderdash_std");
    #[cfg(not(feature = "std"))]
    tenderdash_proto_compiler::proto_compile("tenderdash_nostd");

    println!("cargo:rerun-if-changed=../proto-compiler/src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=TENDERDASH_COMMITISH");
}
