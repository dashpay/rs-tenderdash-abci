# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.8.0] - 2026-09-16

### Changed

- **Default Tenderdash version bumped to `v1.8.0`** (was `v1.5.3`): protobuf definitions are generated from Tenderdash v1.8.0 unless `TENDERDASH_COMMITISH` is set. The crate version is aligned with the targeted Tenderdash version.
- **ABCI protocol version is `1.4.0`** (was `1.3.0`, unchanged in Tenderdash v1.6.x/v1.7.x). `check_version()` requires Tenderdash ABCI `^1.4`, so apps built on this release reject Tenderdash nodes older than v1.8.0 during the `info` handshake.

### Added

- `ResponseFinalizeBlock.propose_next_block_immediately` (`bool`, default `false`): tells the local Tenderdash node to skip the `create-empty-blocks-interval` wait before proposing round 0 of the next height, for example when withdrawal transactions are waiting to be signed. The field is additive, so code that builds `ResponseFinalizeBlock` with `..Default::default()` needs no changes.

### Documentation

- `ValidatorParams.voting_power_threshold` doc comments describe Tenderdash's type-aware threshold validation: an explicit override, the canonical LLMQ threshold, or a size-based floor for custom quorum types. The wire format is unchanged.

[1.8.0]: https://github.com/dashpay/rs-tenderdash-abci/compare/v1.5.1...v1.8.0

## [1.5.1] - 2026-04-24

### Changed

- **MSRV bumped to `1.88`** (was `1.85`) — required by the `zip` 8.5 build-dependency in `tenderdash-proto-compiler`. Consumers of `tenderdash-proto` must build with rustc ≥ 1.88.
- `zip` build-dependency updated from `7.0` to `8.5` in `tenderdash-proto-compiler` (#193).
- Reference Dockerfiles bumped from `rust:1.85-*` to `rust:1.88-*`.

### Fixed

- `tenderdash-proto-compiler`: actionable download-hint message and stale-archive cleanup in the Tenderdash source fetch script (#188).
- `tenderdash-abci` tests: updated `bollard` import to use `bollard::models::ContainerCreateBody` (bollard 0.20.2 dropped the re-export via `bollard::secret::*`).

### CI / Build

- `actions/upload-artifact` 6 → 7 (#183).
- `docker/setup-buildx-action` 3.12.0 → 4.0.0 (#185).
- `docker/build-push-action` 6.18.0 → 7.0.0 (#186) and 7.0.0 → 7.1.0 (#194).
- `actions/deploy-pages` 4 → 5 (#191).
- `actions/configure-pages` 5 → 6 (#192).
- `actions/upload-pages-artifact` 4 → 5 (#195).

[1.5.1]: https://github.com/dashpay/rs-tenderdash-abci/compare/v1.5.0...v1.5.1
