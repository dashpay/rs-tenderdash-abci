use std::env;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    env::set_var("TENDERDASH_COMMITISH", "v".to_owned() + version);
    tenderdash_proto_compiler::proto_compile();

    println!("cargo:rerun-if-changed=../proto-compiler/src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
}
