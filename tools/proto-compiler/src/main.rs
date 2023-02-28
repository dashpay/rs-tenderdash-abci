use std::{env::var, path::PathBuf};

use tempfile::tempdir;

mod functions;
use functions::{copy_files, find_proto_files, generate_tendermint_lib, get_commitish};

mod constants;
use constants::{
    CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, TENDERMINT_COMMITISH, TENDERMINT_REPO,
};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tendermint_lib_target = root
        .join("..")
        .join("..")
        .join("proto")
        .join("src")
        .join("tendermint.rs");
    let target_dir = root
        .join("..")
        .join("..")
        .join("proto")
        .join("src")
        .join("prost");
    let out_dir = var("OUT_DIR")
        .map(PathBuf::from)
        .or_else(|_| tempdir().map(|d| d.into_path()))
        .unwrap();
    let tendermint_dir = PathBuf::from(var("TENDERMINT_DIR").unwrap_or_else(|_| {
        root.join("..")
            .join("target")
            .join("tendermint")
            .to_str()
            .unwrap()
            .to_string()
    }));

    let thirdparty_dir = root.join("third_party");

    println!(
        "[info] => Fetching {TENDERMINT_REPO} at {TENDERMINT_COMMITISH} into {tendermint_dir:?}"
    );
    get_commitish(
        &PathBuf::from(&tendermint_dir),
        TENDERMINT_REPO,
        TENDERMINT_COMMITISH,
    ); // This panics if it fails.

    let proto_paths = vec![tendermint_dir.join("proto")];
    let proto_includes_paths = vec![
        tendermint_dir.join("proto"),
        tendermint_dir.join("third_party").join("proto"),
        thirdparty_dir,
    ];
    // List available proto files
    let protos = find_proto_files(proto_paths);

    let mut pb = prost_build::Config::new();

    // Use shared Bytes buffers for ABCI messages:
    pb.bytes([".tendermint.abci"]);

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
    println!("[info] => Creating structs.");
    pb.compile_protos(&protos, &proto_includes_paths).unwrap();

    println!("[info] => Removing old structs and copying new structs.");
    copy_files(&out_dir, &target_dir); // This panics if it fails.
    generate_tendermint_lib(&out_dir, &tendermint_lib_target);

    println!("[info] => Done!");
}
