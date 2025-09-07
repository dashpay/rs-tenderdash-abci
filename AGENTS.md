# Repository Guidelines

## Project Structure & Module Organization
- `abci/`: `tenderdash-abci` crate (server, application glue). Examples in `abci/examples/` and integration tests in `abci/tests/`.
- `proto/`: `tenderdash-proto` crate (generated types, serializers, gRPC facets).
- `proto-compiler/`: build helpers for protobuf/tonic codegen.
- `scripts/`: release and utility scripts. CI lives under `.github/workflows/`.

## Build, Test, and Development Commands
- Build all (with defaults): `cargo build --workspace --all-features`.
- Test all crates: `cargo test --workspace --all-features`.
- Run example ABCI server: `RUST_LOG=info cargo run -p tenderdash-abci --example echo_socket`.
- Format check: `cargo fmt --all -- --check`; apply fixes with `cargo fmt --all`.
- Lint locally: `cargo clippy --all-features -- -D warnings`.
- Docs: `cargo doc --no-deps --all-features`.

## Coding Style & Naming Conventions
- Rust edition 2024; toolchain `rust-version` is set in workspace.
- Use rustfmt (see `.rustfmt.toml`); CI enforces `cargo fmt --check`.
- Prefer idiomatic Rust: `snake_case` for modules/functions, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Keep imports ordered; the repo config groups std/external/crate imports.

## Testing Guidelines
- Framework: standard Rust tests; async via `tokio` where needed.
- Integration tests live in `abci/tests/*` (some spin up Docker). To skip Docker-dependent tests locally: `cargo test -p tenderdash-abci --no-default-features --features server,tcp,unix`.
- Name tests descriptively; keep unit tests near code and integration tests under `tests/`.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (e.g., `feat:`, `fix:`, `chore:`, `build(deps):`). Use `!` for breaking changes.
- PRs must use the provided template: include motivation, what changed, tests, and note breaking changes. Link related issues.
- Ensure CI passes (fmt, tests, clippy) and add/update tests for new behavior.

## Security & Configuration Tips
- Protobuf codegen requires `protoc` to be installed and discoverable in PATH.
- Default features include `server` and Docker-backed tests; adjust features to your environment.
