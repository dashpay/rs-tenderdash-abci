use std::{
    env,
    path::Path,
    process::{exit, Command},
};

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let version = env!("CARGO_PKG_VERSION");
    let proto_compiler_dir = manifest_dir.join("..").join("tools").join("proto-compiler");

    let output = Command::new("cargo")
        .current_dir(proto_compiler_dir)
        .env("TENDERDASH_COMMITISH", "v".to_owned() + version)
        .arg("run")
        .output()
        .expect("failed to generate protobuf files with Cargo");

    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        exit(output.status.code().unwrap());
    }

    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
}
