# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
