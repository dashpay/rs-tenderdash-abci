# Protobuf compilation

## How to compile fresh proto structs

1. Set env variable `TENDERDASH_COMMITISH` to point to desired git reference (tag, branch, etc.).
2. Run `cargo run` in the compiler folder. The resultant structs will be created in the `proto/src/prost` folder.
3. Build the `tenderdash-proto` crate.

Note: proto structs are also automatically regenerated when building `tenderdash-proto` crate.
