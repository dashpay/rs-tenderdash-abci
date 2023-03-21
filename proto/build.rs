use std::{env, path::Path};

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let version = env!("CARGO_PKG_VERSION");
    // let proto_compiler_dir =
    // manifest_dir.join("..").join("tools").join("proto-compiler");

    env::set_var("TENDERDASH_COMMITISH", "v".to_owned() + version);
    tenderdash_proto_compiler::proto_compile().expect("protobuf definitions compilation failed");

    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
}
