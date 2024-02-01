use std::{env::var, path::PathBuf};

use tempfile::tempdir;

mod functions;
use functions::{
    abci_version, copy_files, fetch_commitish, find_proto_files, generate_tenderdash_lib,
    tenderdash_commitish, tenderdash_version,
};

mod constants;
use constants::{CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, TENDERDASH_REPO};

/// Import and compile protobuf definitions for Tenderdash.
///
/// Checkouts tenderdash repository to ../target/tenderdash and generates
/// Rust protobuf definitions in ../proto/src/prost/ and
/// ../proto/src/tenderdash.rs
pub fn proto_compile() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let tenderdash_lib_target = root
        .join("..")
        .join("proto")
        .join("src")
        .join("tenderdash.rs");

    let target_dir = root.join("..").join("proto").join("src").join("prost");

    let out_dir = var("OUT_DIR")
        .map(PathBuf::from)
        .or_else(|_| tempdir().map(|d| d.into_path()))
        .unwrap();

    let cargo_target_dir = match std::env::var("CARGO_TARGET_DIR") {
        Ok(s) => PathBuf::from(s),
        Err(_) => root.join("..").join("target"),
    };
    let tenderdash_dir = PathBuf::from(var("TENDERDASH_DIR").unwrap_or_else(|_| {
        cargo_target_dir
            .join("tenderdash")
            .to_str()
            .unwrap()
            .to_string()
    }));

    let thirdparty_dir = root.join("third_party");

    let commitish = tenderdash_commitish();
    println!("[info] => Fetching {TENDERDASH_REPO} at {commitish} into {tenderdash_dir:?}");
    fetch_commitish(
        &PathBuf::from(&tenderdash_dir),
        &cargo_target_dir,
        TENDERDASH_REPO,
        &commitish,
    ); // This panics if it fails.

    // We need all files in proto/tendermint/abci, plus .../types/canonical.proto
    // for signature verification
    let proto_paths = vec![tenderdash_dir.join("proto").join("tendermint").join("abci")];
    let proto_includes_paths = vec![tenderdash_dir.join("proto"), thirdparty_dir];
    // List available proto files
    let mut protos = find_proto_files(proto_paths);
    // On top of that, we add canonical.proto, required to verify signatures
    protos.push(
        tenderdash_dir
            .join("proto")
            .join("tendermint")
            .join("types")
            .join("canonical.proto"),
    );

    let mut pb = prost_build::Config::new();

    // Compile proto files with added annotations, exchange prost_types to our own
    pb.out_dir(&out_dir);
    for type_attribute in CUSTOM_TYPE_ATTRIBUTES {
        pb.type_attribute(type_attribute.0, type_attribute.1);
    }
    for field_attribute in CUSTOM_FIELD_ATTRIBUTES {
        pb.field_attribute(field_attribute.0, field_attribute.1);
    }
    // The below in-place path redirection replaces references to the Duration
    // and Timestamp WKTs with our own versions that have valid doctest comments.
    // See also https://github.com/danburkert/prost/issues/374 .
    pb.extern_path(
        ".google.protobuf.Duration",
        "super::super::google::protobuf::Duration",
    );
    pb.extern_path(
        ".google.protobuf.Timestamp",
        "super::super::google::protobuf::Timestamp",
    );

    println!("[info] => Determining ABCI protocol version.");
    let abci_ver = abci_version(&tenderdash_dir);
    let tenderdash_ver = tenderdash_version(tenderdash_dir);

    println!("[info] => Creating structs.");
    pb.compile_protos(&protos, &proto_includes_paths).unwrap();

    println!("[info] => Removing old structs and copying new structs.");
    copy_files(&out_dir, &target_dir); // This panics if it fails.

    generate_tenderdash_lib(&out_dir, &tenderdash_lib_target, &abci_ver, &tenderdash_ver);

    println!("[info] => Done!");
}
