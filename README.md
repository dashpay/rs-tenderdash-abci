# Rust bindings and server for Tenderdash ABCI++ protocol

## Introduction

![example workflow](https://github.com/dashpay/rs-tenderdash-abci/actions/workflows/test.yml/badge.svg?branch=master)

This repository contains Rust bindings for Tenderdash. It includes:

* data types, requests and responses required on [Tenderdash]
* ABCI++ protocol server, supporting **Unix sockets** and **TCP** connections

## Structure

The repository contains the following crates:

* [tenderdash-abci](./abci/) - main crate, including ABCI++ server implementation, `Application` trait and re-exporting `tenderdash-proto` crate
* [tenderdash-proto](./proto/) - ABCI++ messages and data types definitions, generated based on Tenderdash protobuf specifications
* [tenderdash-proto-compiler](./proto-compiler/) - an internal tool that fetches tenderdash repository and converts protobuf files to Rust

## Versioning

The major and Minor version of this library matches the major and minor version of [Tenderdash]. For example, for `Tenderdash 1.2.34`, use `rs-tenderdash-abci 1.2.*`.

You should also check the protocol version in `init_chain()`.

## Quick start

1. Install dependencies. You can find a current list of dependencies in the [Dockerfile](Dockerfile).
2. Add tenderdash-abci crate to your project:

    ```bash
    cargo add --git https://github.com/dashpay/rs-tenderdash-abci tenderdash-abci
    ```

3. Implement the [Application](abci/src/application.rs) trait with your custom logic. You can check [kvstore](abci/tests/kvstore.rs) as a minimal example.

## Credits

This project is a partial fork of [tendermint-rs] project.

## License

[MIT](LICENSE.md)

[Tenderdash]: https://github.com/dashpay/tenderdash
[tendermint-rs]: https://github.com/informalsystems/tendermint-rs
