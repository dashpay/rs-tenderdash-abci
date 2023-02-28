# Protobuf compilation

## How to compile fresh proto structs

1. Ensure that  `TENDERDASH_COMMITISH` in `src/constants.rs` is set to correct git reference (tag or branch).
2. Run `cargo run` in the compiler folder. The resultant structs will be created in the `proto/src/prost` folder.
3. Build the `tenderdash-proto` crate.
