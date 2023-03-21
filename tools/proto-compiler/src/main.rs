use std::{env::var, path::PathBuf};

use tempfile::tempdir;

mod functions;
use functions::{
    abci_version, copy_files, fetch_commitish, find_proto_files, generate_tenderdash_lib,
    tenderdash_commitish,
};

mod constants;
use constants::{CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, TENDERDASH_REPO};
use tenderdash_proto_compiler::proto_compile;

fn main() {
    proto_compile().expect("protobuf definitions compilation failed")
}
